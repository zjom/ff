use crate::ast::Program;
use crate::interpreter::{Env, Scope, Value, eval_program};
use crate::parser::parse;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Config, EditMode, Editor, Completer, Helper, Highlighter, Hinter};

#[derive(Completer, Helper, Highlighter, Hinter)]
struct REPLHelper {}

impl Validator for REPLHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        if input.is_empty() {
            return Ok(ValidationResult::Valid(None));
        }

        // If the input ends with a newline, it means the user pressed Enter on an empty line
        // at the end of a multi-line input, which we treat as a submission/abort.
        if input.ends_with('\n') {
            return Ok(ValidationResult::Valid(None));
        }

        match parse(input) {
            Ok(_) => {
                if ends_with_continuation_comma(input) {
                    Ok(ValidationResult::Incomplete)
                } else {
                    Ok(ValidationResult::Valid(None))
                }
            }
            Err(_) => {
                // Treat parse errors as incomplete to allow for multi-line input.
                Ok(ValidationResult::Incomplete)
            }
        }
    }
}

pub fn run() {
    let env = Scope::new();
    let config = Config::builder()
        .edit_mode(EditMode::Emacs)
        .build();
    let helper = REPLHelper {};
    let mut rl = Editor::with_config(config).unwrap();
    rl.set_helper(Some(helper));

    // Try to load history from a local file
    let history_path = ".ff_history";
    let _ = rl.load_history(history_path);

    println!("ff repl — Ctrl-D to exit, blank line to submit/abort multi-line input");
    loop {
        let prompt = ">> ";
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                rl.add_history_entry(line.as_str()).ok();
                
                match parse(&line) {
                    Ok(program) => {
                        run_program(&program, &env);
                    }
                    Err(e) => {
                        eprintln!("parse error: {}", e);
                    }
                }
                // Save history after each successful command
                let _ = rl.save_history(history_path);
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: Clear the current buffer and start over
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(e) => {
                eprintln!("error: {}", e);
                break;
            }
        }
    }
}

fn run_program(program: &Program, env: &Env) {
    match eval_program(program, env) {
        Ok(Value::Unit) => {}
        Ok(v) => println!("{}", v),
        Err(e) => eprintln!("error: {}", e),
    }
}

// Scan past whitespace, newlines, comments, and string literals to find the
// last syntactically meaningful character. Strings have no escape sequences
// in this grammar, so `"..."` matching is sufficient.
fn ends_with_continuation_comma(s: &str) -> bool {
    let mut last: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' => {
                while let Some(&n) = chars.peek() {
                    if n == '\n' || n == '\r' {
                        break;
                    }
                    chars.next();
                }
            }
            '"' => {
                last = Some('"');
                for c2 in chars.by_ref() {
                    if c2 == '"' {
                        break;
                    }
                }
            }
            c if c.is_whitespace() => {}
            c => last = Some(c),
        }
    }
    last == Some(',')
}
