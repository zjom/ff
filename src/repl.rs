use crate::interpreter::{Scope, Value, eval_program};
use crate::parser::parse;
use std::io::{self, BufRead, Write};

pub fn run() {
    let env = Scope::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    let mut buf = String::new();
    let mut continuing = false;

    println!("ff repl — Ctrl-D to exit, blank line to abort multi-line input");
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
                Err(e) => eprintln!("parse error: {}", e),
                Ok(_) => unreachable!("buffer parses but we were still continuing"),
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
                match eval_program(&program, &env) {
                    Ok(Value::Unit) => {}
                    Ok(v) => println!("{}", v),
                    Err(e) => eprintln!("error: {}", e),
                }
                buf.clear();
                continuing = false;
            }
            Err(_) => {
                continuing = true;
            }
        }
    }
}
