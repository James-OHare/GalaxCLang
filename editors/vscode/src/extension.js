const vscode = require('vscode');
const { exec } = require('child_process');

/**
 * Activates the GalaxC language extension.
 * Provides on-save diagnostic checking via the 'galaxc check' command.
 */
function activate(context) {
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('galaxc');
    context.subscriptions.push(diagnosticCollection);

    // Watch for document saves to trigger check
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((document) => {
            if (document.languageId === 'galaxc') {
                runGalaxCCheck(document, diagnosticCollection);
            }
        })
    );

    // Watch for document opens to trigger initial check
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((document) => {
            if (document.languageId === 'galaxc') {
                runGalaxCCheck(document, diagnosticCollection);
            }
        })
    );

    // Initial check for currently active editor
    if (vscode.window.activeTextEditor && vscode.window.activeTextEditor.document.languageId === 'galaxc') {
        runGalaxCCheck(vscode.window.activeTextEditor.document, diagnosticCollection);
    }

    console.log('GalaxC language support activated with real-time diagnostics.');

    // Provide document symbols for the Outline view
    context.subscriptions.push(
        vscode.languages.registerDocumentSymbolProvider(
            { language: 'galaxc' },
            new GalaxCSymbolProvider()
        )
    );

    // Command: Run Current File
    context.subscriptions.push(
        vscode.commands.registerCommand('galaxc.runFile', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'galaxc') {
                const terminal = vscode.window.createTerminal('GalaxC Run');
                terminal.show();
                terminal.sendText(`galaxc run "${editor.document.fileName}"`);
            }
        })
    );

    // Command: Build Current File
    context.subscriptions.push(
        vscode.commands.registerCommand('galaxc.buildFile', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'galaxc') {
                const terminal = vscode.window.createTerminal('GalaxC Build');
                terminal.show();
                terminal.sendText(`galaxc build "${editor.document.fileName}"`);
            }
        })
    );

    // Command: Create New File
    context.subscriptions.push(
        vscode.commands.registerCommand('galaxc.createFile', async () => {
            const uri = await vscode.window.showSaveDialog({
                defaultUri: vscode.Uri.file('program.gxc'),
                filters: { 'GalaxC': ['gxc'] }
            });
            if (uri) {
                const content = 'orbit main\n\n@effect(io)\nop launch() =>\n    console.write("Hello, World!")\nend\n';
                await vscode.workspace.fs.writeFile(uri, Buffer.from(content));
                const doc = await vscode.workspace.openTextDocument(uri);
                await vscode.window.showTextDocument(doc);
            }
        })
    );

    // Command: Init Project
    context.subscriptions.push(
        vscode.commands.registerCommand('galaxc.initProject', async (uri) => {
            const name = await vscode.window.showInputBox({ 
                prompt: 'Enter project name',
                placeHolder: 'mission_alpha'
            });
            if (name) {
                const terminal = vscode.window.createTerminal('GalaxC Init');
                terminal.show();
                terminal.sendText(`galaxc init ${name}`);
            }
        })
    );

    // Status bar item for run
    const runBtn = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    runBtn.command = 'galaxc.runFile';
    runBtn.text = '$(play) Run GalaxC';
    runBtn.tooltip = 'Compile and run current GalaxC file';
    context.subscriptions.push(runBtn);

    context.subscriptions.push(vscode.window.onDidChangeActiveTextEditor(e => {
        if (e && e.document.languageId === 'galaxc') {
            runBtn.show();
        } else {
            runBtn.hide();
        }
    }));
    if (vscode.window.activeTextEditor?.document.languageId === 'galaxc') {
        runBtn.show();
    }
}

/**
 * Parses the document to find operations, structs, and tasks for the Outline view.
 */
class GalaxCSymbolProvider {
    provideDocumentSymbols(document, token) {
        return new Promise((resolve) => {
            const symbols = [];
            for (let i = 0; i < document.lineCount; i++) {
                const line = document.lineAt(i);
                
                // Match "op name"
                const opMatch = line.text.match(/^\s*op\s+([a-z_][a-z0-9_]*)/);
                if (opMatch) {
                    symbols.push(new vscode.DocumentSymbol(
                        opMatch[1],
                        'Operation',
                        vscode.SymbolKind.Function,
                        line.range,
                        line.range
                    ));
                }

                // Match "struct Name"
                const structMatch = line.text.match(/^\s*struct\s+([A-Z][a-zA-Z0-9]*)/);
                if (structMatch) {
                    symbols.push(new vscode.DocumentSymbol(
                        structMatch[1],
                        'Struct',
                        vscode.SymbolKind.Struct,
                        line.range,
                        line.range
                    ));
                }

                // Match "task Name"
                const taskMatch = line.text.match(/^\s*task\s+([A-Z][a-zA-Z0-9]*)/);
                if (taskMatch) {
                    symbols.push(new vscode.DocumentSymbol(
                        taskMatch[1],
                        'Task',
                        vscode.SymbolKind.Class,
                        line.range,
                        line.range
                    ));
                }

                // Match "enum Name"
                const enumMatch = line.text.match(/^\s*enum\s+([A-Z][a-zA-Z0-9]*)/);
                if (enumMatch) {
                    symbols.push(new vscode.DocumentSymbol(
                        enumMatch[1],
                        'Enum',
                        vscode.SymbolKind.Enum,
                        line.range,
                        line.range
                    ));
                }
            }
            resolve(symbols);
        });
    }
}

/**
 * Executes 'galaxc check' and parses output into VS Code diagnostics.
 */
function runGalaxCCheck(document, collection) {
    const filePath = document.fileName;
    // We use the 'galaxc' command installed in the user's path.
    exec(`galaxc check "${filePath}"`, (error, stdout, stderr) => {
        collection.delete(document.uri);
        const diagnostics = [];
        const output = stdout + stderr;
        const lines = output.split(/\r?\n/);

        let currentMsg = null;
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            // Match error message: "error: [msg]"
            const errorMatch = line.match(/^error: (.+)$/);
            if (errorMatch) {
                currentMsg = errorMatch[1];
                continue;
            }

            // Match location: "  --> file:line:col"
            const locMatch = line.match(/^\s*--> .+:(\d+):(\d+)$/);
            if (locMatch && currentMsg) {
                const lineIdx = parseInt(locMatch[1]) - 1;
                const colIdx = parseInt(locMatch[2]) - 1;

                // Create range (default to 1 character if we can't find exact span)
                const range = new vscode.Range(
                    lineIdx, colIdx, 
                    lineIdx, colIdx + 1
                );

                const diagnostic = new vscode.Diagnostic(
                    range,
                    currentMsg,
                    vscode.DiagnosticSeverity.Error
                );
                diagnostic.source = 'galaxc';
                diagnostics.push(diagnostic);
                currentMsg = null;
            }
        }
        collection.set(document.uri, diagnostics);
    });
}

function deactivate() {}

module.exports = {
    activate,
    deactivate
};
