mod common;
use common::eval;

#[test]
fn integer_literal() {
    assert_eq!(eval("42"), "42");
}

#[test]
fn decimal_literal() {
    assert_eq!(eval("2.5"), "2.5");
}

#[test]
fn string_literal() {
    assert_eq!(eval(r#""hello""#), r#""hello""#);
}

#[test]
fn bool_literals() {
    assert_eq!(eval("true"), "true");
    assert_eq!(eval("false"), "false");
}

#[test]
fn comments_ignored() {
    assert_eq!(eval("# just a comment\n42 # trailing"), "42");
}

#[test]
fn arithmetic_precedence() {
    assert_eq!(eval("1 + 2 * 3"), "7");
    assert_eq!(eval("(1 + 2) * 3"), "9");
}

#[test]
fn power_right_associative() {
    assert_eq!(eval("2 ** 3 ** 2"), "512");
}

#[test]
fn integer_modulo() {
    assert_eq!(eval("10 % 3"), "1");
}

#[test]
fn unary_neg_and_not() {
    assert_eq!(eval("-(3 + 4)"), "-7");
    assert_eq!(eval("!true"), "false");
    assert_eq!(eval("!!true"), "true");
}

#[test]
fn string_concatenation() {
    assert_eq!(eval(r#""foo" :: "bar""#), r#""foobar""#);
}

#[test]
fn comparisons_on_numbers() {
    assert_eq!(eval("1 < 2"), "true");
    assert_eq!(eval("2 <= 2"), "true");
    assert_eq!(eval("3 > 2"), "true");
    assert_eq!(eval("2 >= 2"), "true");
    assert_eq!(eval("1 == 1"), "true");
    assert_eq!(eval("1 != 2"), "true");
}

#[test]
fn comparisons_on_strings() {
    assert_eq!(eval(r#""abc" < "abd""#), "true");
    assert_eq!(eval(r#""x" == "x""#), "true");
}

#[test]
fn structural_equality() {
    assert_eq!(eval("[1, 2] == [1, 2]"), "true");
    assert_eq!(eval("[1, 2] == [1, 3]"), "false");
    assert_eq!(eval(r#"{"a": 1} == {"a": 1}"#), "true");
    assert_eq!(eval("{1, 2, 3} == {3, 2, 1}"), "true");
}

#[test]
fn match_operator_substring() {
    assert_eq!(eval(r#""hello world" ~ "world""#), "true");
    assert_eq!(eval(r#""hello" ~ "xyz""#), "false");
    assert_eq!(eval(r#""hello" !~ "xyz""#), "true");
}

#[test]
fn logical_ops_short_circuit() {
    assert_eq!(eval("true && false"), "false");
    assert_eq!(eval("true || false"), "true");
    // Right side never evaluated; if it were, `undef` would error.
    assert_eq!(eval("false && undef"), "false");
    assert_eq!(eval("true || undef"), "true");
}

#[test]
fn if_then_else_truthy() {
    assert_eq!(eval("if 3 > 2 then \"yes\" else \"no\""), "\"yes\"");
}

#[test]
fn if_then_else_falsy() {
    assert_eq!(eval("if 1 > 2 then 0 else -1"), "-1");
}

#[test]
fn if_multiline_clause_breaks() {
    let src = "if 3 > 2\nthen \"yes\"\nelse \"no\"";
    assert_eq!(eval(src), "\"yes\"");
}

#[test]
fn if_multiline_after_then_and_else() {
    let src = "if 1 > 2 then\n  0\nelse\n  -1";
    assert_eq!(eval(src), "-1");
}

#[test]
fn if_multiline_indented_clauses() {
    let src = "x = if true\n  then 10\n  else 20\nx";
    assert_eq!(eval(src), "10");
}

#[test]
fn assignment_sequence() {
    assert_eq!(eval("x = 10\ny = 20\nx + y"), "30");
}

#[test]
fn rebinding() {
    assert_eq!(eval("x = 1\nx = x + 1\nx = x + 1\nx"), "3");
}

#[test]
fn atom_literal() {
    assert_eq!(eval(":ok"), ":ok");
    assert_eq!(eval(":hello_world"), ":hello_world");
    assert_eq!(eval(":_priv"), ":_priv");
}

#[test]
fn atom_equality() {
    assert_eq!(eval(":ok == :ok"), "true");
    assert_eq!(eval(":ok == :error"), "false");
    assert_eq!(eval(":ok != :error"), "true");
    // Atoms are their own type — never equal to a string of the same name.
    assert_eq!(eval(r#":ok == "ok""#), "false");
}

#[test]
fn atom_in_collections() {
    assert_eq!(eval("[:a, :b, :c]"), "[:a, :b, :c]");
    assert_eq!(eval("[:ok, 1]"), "[:ok, 1]");
    assert_eq!(eval("{:a, :b, :a}"), "{:a, :b}");
    // Objects are unordered; display sorts entries by key.
    assert_eq!(eval("{:ok: 1, :error: 2}"), "{:error: 2, :ok: 1}");
}

#[test]
fn dot_atom_object_access() {
    assert_eq!(eval("{:a: 1, :b: 2}.a"), "1");
    assert_eq!(eval("{:ok: 42}.ok"), "42");
}
