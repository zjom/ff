use f2::interop::{define_value, from_value, register, to_value};
use f2::interpreter::{Scope, Value, eval_program};
use f2::parser::parse;
use f2::prelude;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Person {
    name: String,
    age: u32,
    tags: Vec<String>,
}

fn fresh_env() -> f2::interpreter::Env {
    let env = Scope::new();
    prelude::install(&env);
    env
}

fn run(env: &f2::interpreter::Env, src: &str) -> Value {
    let prog = parse(src).expect("parse");
    eval_program(&prog, env).expect("eval")
}

#[test]
fn round_trip_struct() {
    let p = Person {
        name: "Ada".into(),
        age: 36,
        tags: vec!["math".into(), "cs".into()],
    };
    let v = to_value(&p).unwrap();
    let back: Person = from_value(v).unwrap();
    assert_eq!(p, back);
}

#[test]
fn pass_struct_into_script() {
    let env = fresh_env();
    let p = Person {
        name: "Ada".into(),
        age: 36,
        tags: vec!["math".into()],
    };
    define_value(&env, "person", &p).unwrap();
    let out = run(&env, r#"person.name :: " is " :: person.tags.0"#);
    assert_eq!(out.to_string(), r#""Ada is math""#);
}

#[test]
fn call_unary_rust_fn() {
    let env = fresh_env();
    register(&env, "double", |x: i64| x * 2);
    let out = run(&env, "double(21)");
    assert_eq!(out.to_string(), "42");
}

#[test]
fn call_binary_rust_fn_curried() {
    let env = fresh_env();
    register(&env, "add", |a: i64, b: i64| a + b);
    assert_eq!(run(&env, "add(3, 4)").to_string(), "7");
    assert_eq!(run(&env, "add(3)(4)").to_string(), "7");
}

#[test]
fn rust_fn_returning_unit() {
    let env = fresh_env();
    register(&env, "noop", || ());
    let out = run(&env, "noop()");
    assert_eq!(out, Value::Unit);
}

#[test]
fn rust_fn_returning_struct() {
    let env = fresh_env();
    register(&env, "make_person", |name: String, age: u32| Person {
        name,
        age,
        tags: vec![],
    });
    let out = run(&env, r#"make_person("Ada", 36).name"#);
    assert_eq!(out, Value::String("Ada".into()));
}

#[test]
fn rust_fn_takes_struct() {
    let env = fresh_env();
    register(&env, "greet", |p: Person| format!("Hi {}!", p.name));
    let out = run(&env, r#"greet({"name": "Ada", "age": 36, "tags": []})"#);
    assert_eq!(out, Value::String("Hi Ada!".into()));
}

#[test]
fn rust_fn_returning_vec() {
    let env = fresh_env();
    register(&env, "range_doubled", |n: u32| {
        (0..n).map(|i| i * 2).collect::<Vec<_>>()
    });
    assert_eq!(run(&env, "range_doubled(4)").to_string(), "[0, 2, 4, 6]");
}

#[test]
fn closure_captures_state() {
    let env = fresh_env();
    let secret: u32 = 99;
    register(&env, "secret", move || secret);
    assert_eq!(run(&env, "secret()").to_string(), "99");
}

#[test]
fn negative_integer_round_trip() {
    let v = to_value(&-7i64).unwrap();
    let back: i64 = from_value(v).unwrap();
    assert_eq!(back, -7);
}

#[test]
fn float_round_trip() {
    let v = to_value(&1.5f64).unwrap();
    let back: f64 = from_value(v).unwrap();
    assert_eq!(back, 1.5);
}

#[test]
fn vec_round_trip() {
    let v = to_value(&vec![1u32, 2, 3]).unwrap();
    let back: Vec<u32> = from_value(v).unwrap();
    assert_eq!(back, vec![1, 2, 3]);
}

#[test]
fn option_some_and_none() {
    let some: Option<i32> = Some(5);
    let none: Option<i32> = None;
    assert_eq!(
        from_value::<Option<i32>>(to_value(&some).unwrap()).unwrap(),
        some
    );
    assert_eq!(
        from_value::<Option<i32>>(to_value(&none).unwrap()).unwrap(),
        none
    );
}

#[test]
fn atom_round_trim() {
    let v = to_value(":atom").unwrap();
    let back: String = from_value(v).unwrap();
    assert_eq!(back, ":atom");
}
