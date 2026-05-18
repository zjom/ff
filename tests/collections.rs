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
fn unit_literal() {
    assert_eq!(eval("()"), "()");
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

// --- multiline collections --------------------------------------------------

#[test]
fn list_newline_separated() {
    assert_eq!(eval("[\n1\n2\n3\n]"), "[1, 2, 3]");
}

#[test]
fn list_newline_after_comma() {
    assert_eq!(eval("[1,\n2,\n3]"), "[1, 2, 3]");
}

#[test]
fn list_extra_commas_and_newlines_collapse() {
    // Any non-empty run of `,` and newlines counts as a single separator.
    assert_eq!(eval("[1,\n   ,2]"), "[1, 2]");
}

#[test]
fn list_leading_and_trailing_separators() {
    assert_eq!(eval("[\n  ,1,\n  2,\n]"), "[1, 2]");
}

#[test]
fn dict_multiline() {
    assert_eq!(
        eval("{\n  \"a\": 1,\n  \"b\": 2\n}"),
        r#"{"a": 1, "b": 2}"#
    );
}

#[test]
fn dict_value_on_next_line() {
    assert_eq!(eval("{\"a\":\n  1}"), r#"{"a": 1}"#);
}

#[test]
fn set_multiline() {
    assert_eq!(eval("{\n1\n2\n3\n}"), "{1, 2, 3}");
}

#[test]
fn call_args_multiline() {
    assert_eq!(eval("f = (a, b, c) => a + b + c\nf(\n  1,\n  2,\n  3,\n)"), "6");
}
