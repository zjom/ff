mod common;
use common::eval;

// --- destructuring assignment -----------------------------------------------

#[test]
fn destructure_list_exact() {
    assert_eq!(eval("[a, b, c] = [1, 2, 3]\nb"), "2");
}

#[test]
fn destructure_with_rest() {
    assert_eq!(eval("[head, ..tail] = [1, 2, 3, 4]\ntail"), "[2, 3, 4]");
}

#[test]
fn destructure_rest_in_middle() {
    assert_eq!(eval("[first, ..mid, last] = [1, 2, 3, 4, 5]\nmid"), "[2, 3, 4]");
    assert_eq!(eval("[first, ..mid, last] = [1, 2, 3, 4, 5]\nlast"), "5");
}

#[test]
fn destructure_anonymous_rest() {
    assert_eq!(eval("[head, ..] = [10, 20, 30]\nhead"), "10");
}

#[test]
fn destructure_tuple() {
    assert_eq!(eval("(x, y) = (1, 2)\nx + y"), "3");
}

#[test]
fn destructure_dict() {
    assert_eq!(eval(r#"{"name": who} = {"name": "ada", "age": 36}
who"#), r#""ada""#);
}

#[test]
fn destructure_nested() {
    assert_eq!(eval("[[a, b], [c, d]] = [[1, 2], [3, 4]]\na + d"), "5");
}

// --- match expression -------------------------------------------------------

#[test]
fn match_literal_arm() {
    assert_eq!(eval("match 1\n  0 -> \"zero\",\n  1 -> \"one\",\n  n -> \"other\""), "\"one\"");
}

#[test]
fn match_binding_arm() {
    assert_eq!(eval("match 99\n  0 -> 0,\n  n -> n * 2"), "198");
}

#[test]
fn match_wildcard_arm() {
    assert_eq!(eval("match 42\n  0 -> \"zero\",\n  _ -> \"other\""), "\"other\"");
}

#[test]
fn match_list_pattern() {
    assert_eq!(eval("match [1, 2, 3]\n  [] -> 0,\n  [x] -> 1,\n  _ -> 99"), "99");
}

#[test]
fn match_list_rest() {
    assert_eq!(
        eval("match [1, 2, 3, 4]\n  [x, ..rest] -> rest"),
        "[2, 3, 4]"
    );
}

#[test]
fn match_dict_pattern() {
    // Dict pattern matches if all required keys exist; extras allowed.
    assert_eq!(
        eval(r#"match {"name": "ada", "age": 36}
  {"name": n} -> n"#),
        r#""ada""#
    );
}

#[test]
fn match_trailing_comma() {
    assert_eq!(eval("match 1\n  0 -> \"zero\",\n  1 -> \"one\",\n"), "\"one\"");
}

#[test]
fn match_arms_inline_with_commas() {
    assert_eq!(eval("match 2\n  1 -> \"a\", 2 -> \"b\", _ -> \"c\""), "\"b\"");
}

#[test]
fn match_missing_comma_is_parse_error() {
    // Newlines no longer separate arms; missing comma must fail to parse.
    assert!(ff::parser::parse("match 1\n  0 -> \"zero\"\n  1 -> \"one\"").is_err());
}

#[test]
fn match_pattern_failure_is_runtime_error() {
    // None of the arms match → runtime error (we test the error path elsewhere).
    let src = "match 99\n  0 -> 0,\n  1 -> 1";
    let prog = ff::parser::parse(src).expect("parse");
    assert!(ff::interpreter::run(&prog).is_err());
}

// --- match-as-function ------------------------------------------------------

#[test]
fn match_as_function_via_call() {
    assert_eq!(
        eval("f = match\n  0 -> \"zero\",\n  n -> \"other\"\nf(0)"),
        r#""zero""#
    );
}

#[test]
fn match_as_function_via_juxt() {
    assert_eq!(
        eval("f = match\n  0 -> \"zero\",\n  n -> \"other\"\nf 5"),
        r#""other""#
    );
}

#[test]
fn match_as_function_recursive() {
    assert_eq!(
        eval("fact = match\n  0 -> 1,\n  n -> n * fact(n - 1)\nfact 6"),
        "720"
    );
}

#[test]
fn match_as_function_list_dispatch() {
    let src = "describe = match\n  [] -> \"empty\",\n  [x] -> \"one\",\n  _ -> \"many\"\ndescribe [1, 2, 3]";
    assert_eq!(eval(src), r#""many""#);
}
