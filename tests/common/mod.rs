#![allow(dead_code)]
use ff::interpreter::run;
use ff::parser::parse;

pub fn eval(src: &str) -> String {
    let prog =
        parse(src).unwrap_or_else(|e| panic!("parse failed: {}\n--- source ---\n{}", e, src));
    let v = run(&prog).unwrap_or_else(|e| panic!("eval failed: {}\n--- source ---\n{}", e, src));
    format!("{}", v)
}

pub fn eval_err(src: &str) -> String {
    let prog =
        parse(src).unwrap_or_else(|e| panic!("parse failed: {}\n--- source ---\n{}", e, src));
    match run(&prog) {
        Ok(v) => panic!(
            "expected eval error, got value {}\n--- source ---\n{}",
            v, src
        ),
        Err(e) => e.to_string(),
    }
}

pub fn parse_err(src: &str) -> String {
    match parse(src) {
        Ok(_) => panic!("expected parse error\n--- source ---\n{}", src),
        Err(e) => e.to_string(),
    }
}
