use galaxc::lexer::tokenize;
use galaxc::parser::parse;
use galaxc::checker::TypeChecker;
use galaxc::diagnostics::Diagnostic;

fn check_code(code: &str) -> Vec<Diagnostic> {
    let tokens = tokenize(code, "test.gxc").expect("Lexer error in test");
    let program = parse(tokens, code, "test.gxc").expect("Parser error in test");
    let checker = TypeChecker::check(&program);
    checker.take_errors()
}

#[test]
fn test_immutable_assignment() {
    let code = "
        orbit test
        op main() =>
            let x = 10
            x = 20
        end
    ";
    let errors = check_code(code);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("cannot assign to immutable"));
    assert_eq!(errors[0].error_code, Some("E0004".to_string()));
}

#[test]
fn test_must_use_result() {
    let code = "
        orbit test
        op main() =>
            let res: Result<Int, Text> = Ok(10)
            res
        end
    ";
    let errors = check_code(code);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("must be handled"));
}

#[test]
fn test_effect_safety() {
    let code = "
        orbit test
        @effect(io)
        op do_io() => end

        op main() =>
            do_io()
        end
    ";
    let errors = check_code(code);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("requires effect 'io'"));
    assert_eq!(errors[0].error_code, Some("E0005".to_string()));
}

#[test]
fn test_match_exhaustiveness() {
    let code = "
        orbit test
        enum State =>
            A
            B
        end
        op main(s: State) =>
            match s =>
                State::A => 1
            end
        end
    ";
    let errors = check_code(code);
    assert!(!errors.is_empty());
    assert!(errors[0].message.contains("variant 'B' not covered"));
    assert_eq!(errors[0].error_code, Some("E0006".to_string()));
}
