mod common;
use common::eval;

#[test]
fn empty_list() {
    assert_eq!(eval("[]"), "[]");
}

#[test]
fn list_literal() {
    assert_eq!(eval("[1, 2, 3]"), "[1, 2, 3]");
}

#[test]
fn list_trailing_comma() {
    assert_eq!(eval("[1, 2, 3,]"), "[1, 2, 3]");
}

#[test]
fn list_index() {
    assert_eq!(eval("[10, 20, 30].0"), "10");
    assert_eq!(eval("[10, 20, 30].2"), "30");
}

#[test]
fn empty_tuple() {
    assert_eq!(eval("()"), "()");
}

#[test]
fn singleton_tuple() {
    assert_eq!(eval("(7,)"), "(7,)");
}

#[test]
fn tuple_literal() {
    assert_eq!(eval("(1, 2, 3)"), "(1, 2, 3)");
}

#[test]
fn tuple_index() {
    assert_eq!(eval("(100, 200).1"), "200");
}

#[test]
fn empty_dict_through_var() {
    // Empty `{}` is ambiguous in pattern position; constructor side parses to dict.
    assert_eq!(eval("d = {}\nd"), "{}");
}

#[test]
fn dict_literal() {
    assert_eq!(eval(r#"{"a": 1, "b": 2}"#), r#"{"a": 1, "b": 2}"#);
}

#[test]
fn dict_field_access() {
    assert_eq!(eval(r#"d = {"name": "ada", "age": 36}
d.name"#), r#""ada""#);
}

#[test]
fn dict_last_write_wins() {
    assert_eq!(eval(r#"{"k": 1, "k": 2}.k"#), "2");
}

#[test]
fn set_deduplicates() {
    assert_eq!(eval("{1, 2, 2, 3, 1}"), "{1, 2, 3}");
}

#[test]
fn nested_dot_access() {
    assert_eq!(eval("[[1, 2], [3, 4]].0.1"), "2");
    assert_eq!(eval("[[1, 2], [3, 4]].1.0"), "3");
}

#[test]
fn dict_into_list_access() {
    assert_eq!(eval(r#"{"items": [10, 20, 30]}.items.1"#), "20");
}

#[test]
fn decimal_index_chains_not_decimal() {
    // `.0.1` must chain as `(.0).1`, never read as the decimal 0.1.
    assert_eq!(eval("[[1, 2], [3, 4]].0.1"), "2");
}
