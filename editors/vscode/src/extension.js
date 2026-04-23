const vscode = require('vscode');
const { exec } = require('child_process');

/**
 * Activates the GalaxC language extension.
 */
function activate(context) {
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('galaxc');
    context.subscriptions.push(diagnosticCollection);

    // Diagnostics (Throttled)
    let timeout;
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((doc) => {
            if (doc.languageId === 'galaxc') runGalaxCCheck(doc, diagnosticCollection);
        }),
        vscode.workspace.onDidChangeTextDocument((e) => {
            if (e.document.languageId === 'galaxc') {
                clearTimeout(timeout);
                timeout = setTimeout(() => runGalaxCCheck(e.document, diagnosticCollection), 1000);
            }
        })
    );
    if (vscode.window.activeTextEditor?.document.languageId === 'galaxc') {
        runGalaxCCheck(vscode.window.activeTextEditor.document, diagnosticCollection);
    }

    // Symbols (Outline)
    context.subscriptions.push(
        vscode.languages.registerDocumentSymbolProvider({ language: 'galaxc' }, new GalaxCSymbolProvider())
    );

    // Commands
    context.subscriptions.push(
        vscode.commands.registerCommand('galaxc.checkFile', () => {
            if (vscode.window.activeTextEditor) runGalaxCCheck(vscode.window.activeTextEditor.document, diagnosticCollection);
        }),
        vscode.commands.registerCommand('galaxc.runFile', () => runInTerminal('Run', 'run')),
        vscode.commands.registerCommand('galaxc.buildFile', () => runInTerminal('Build', 'build')),
        vscode.commands.registerCommand('galaxc.debugFile', () => runInTerminal('Debug', 'debug')),
        vscode.commands.registerCommand('galaxc.initProject', initProject),
        vscode.commands.registerCommand('galaxc.createFile', createFile)
    );

    // Status Bar
    const runBtn = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    runBtn.command = 'galaxc.runFile';
    runBtn.text = '$(play) Run GalaxC';
    runBtn.show();
    context.subscriptions.push(runBtn);
}

function runInTerminal(name, subcmd) {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'galaxc') return;
    const terminal = vscode.window.createTerminal(`GalaxC ${name}`);
    terminal.show();
    const bin = subcmd === 'debug' ? 'galaxc-dbg' : 'galaxc';
    const args = subcmd === 'debug' ? '' : subcmd;
    terminal.sendText(`${bin} ${args} "${editor.document.fileName}"`);
}

async function createFile() {
    const uri = await vscode.window.showSaveDialog({ filters: { 'GalaxC': ['gxc'] } });
    if (uri) {
        const template = 'orbit main\n\n@effect(io)\nop launch() =>\n    console.write("Hello, World!")\nend\n';
        await vscode.workspace.fs.writeFile(uri, Buffer.from(template));
        vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri));
    }
}

async function initProject() {
    const name = await vscode.window.showInputBox({ prompt: 'Project Name' });
    if (name) {
        const terminal = vscode.window.createTerminal('GalaxC Init');
        terminal.show();
        terminal.sendText(`galaxc init ${name}`);
    }
}

function runGalaxCCheck(document, collection) {
    // Attempt to run from path
    exec(`galaxc check "${document.fileName}"`, (err, stdout, stderr) => {
        if (err && err.code === 'ENOENT') {
            vscode.window.showErrorMessage("'galaxc' compiler not found in PATH. Please install it with 'cargo install --path crates/galaxc-cli'.");
            return;
        }

        collection.delete(document.uri);
        const output = (stdout + stderr).replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '');
        const diagnostics = [];
        let currentMsg = null;
        let currentSev = vscode.DiagnosticSeverity.Error;

        output.split(/\r?\n/).forEach(line => {
            const raw = line.trim();
            // Match "error: message" or "warning: message"
            const m = raw.match(/^(error|warning): (.+)$/i);
            if (m) {
                currentMsg = m[2];
                currentSev = m[1].toLowerCase() === 'warning' ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;
            } else {
                // Match "--> file:line:col" (handle DRIVE:\path and relative paths)
                const l = raw.match(/^-->\s+(.+):(\d+):(\d+)$/);
                if (l && currentMsg) {
                    const lineIdx = parseInt(l[2]) - 1;
                    const colIdx = parseInt(l[3]) - 1;
                    const range = new vscode.Range(lineIdx, colIdx, lineIdx, colIdx + 5);
                    diagnostics.push(new vscode.Diagnostic(range, currentMsg, currentSev));
                    currentMsg = null;
                }
            }
        });
        collection.set(document.uri, diagnostics);
    });
}

class GalaxCSymbolProvider {
    provideDocumentSymbols(document) {
        const symbols = [];
        for (let i = 0; i < document.lineCount; i++) {
            const line = document.lineAt(i).text;
            const m = line.match(/^\s*(op|struct|enum|task)\s+([a-zA-Z_][a-zA-Z0-9_]*)/);
            if (m) {
                const kind = m[1] === 'op' ? vscode.SymbolKind.Function : m[1] === 'task' ? vscode.SymbolKind.Class : vscode.SymbolKind.Struct;
                symbols.push(new vscode.DocumentSymbol(m[2], m[1], kind, document.lineAt(i).range, document.lineAt(i).range));
            }
        }
        return symbols;
    }
}

function deactivate() {}
module.exports = { activate, deactivate };
