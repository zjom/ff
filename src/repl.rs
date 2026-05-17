use crate::ast::Program;
use crate::interpreter::{Env, Scope, Value, eval_program};
use crate::parser::parse;
use std::io::{self, BufRead, Write};

pub fn run() {
    let env = Scope::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    let mut buf = String::new();
    let mut continuing = false;

    println!("ff repl — Ctrl-D to exit, blank line to submit/abort multi-line input");
    loop {
        let prompt = if continuing { ".. " } else { ">> " };
        write!(stdout, "{}", prompt).ok();
        stdout.flush().ok();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {}", e);
                break;
            }
        }

        let line = line.trim_end_matches(['\n', '\r']);

        if line.is_empty() {
            if !continuing {
                continue;
            }
            match parse(&buf) {
                Ok(program) => run_program(&program, &env),
                Err(e) => eprintln!("parse error: {}", e),
            }
            buf.clear();
            continuing = false;
            continue;
        }

        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(line);

        match parse(&buf) {
            Ok(program) => {
                // A trailing `,` (e.g. after a match arm) means the user may
                // still be adding more — keep collecting until they submit
                // with a blank line.
                if ends_with_continuation_comma(&buf) {
                    continuing = true;
                } else {
                    run_program(&program, &env);
                    buf.clear();
                    continuing = false;
                }
            }
            Err(_) => {
                continuing = true;
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
