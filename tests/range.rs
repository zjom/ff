mod common;
use common::{eval, parse_err};

#[test]
fn range_is_valid_syntax() {
    assert_eq!(eval("[0..]"), "[0..]");
}

#[test]
fn range_is_invalid_assignment() {
    assert!(!parse_err("[0..] = [1,2,3]").is_empty());
}

// `::` is non-strict in its tail: the rhs is captured as a thunk and only
// forced when something peels into it (pattern match, equality, display).

#[test]
fn cons_is_lazy_in_tail() {
    // If `::` evaluated its right operand strictly, this would diverge on the
    // recursive call into `loop()`. Pulling the head out only forces one step.
    let src = "
loop = () => loop()
match 1 :: loop()
  h :: _ -> h
";
    assert_eq!(eval(src), "1");
}

#[test]
fn map_on_infinite_range_yields_head() {
    let src = "match map(x => x * 2, [0..])
  h :: _ -> h";
    assert_eq!(eval(src), "0");
}

#[test]
fn map_on_infinite_range_peels_multiple() {
    let src = "match map(x => x * 2, [0..])
  a :: b :: c :: _ -> [a, b, c]";
    assert_eq!(eval(src), "[0, 2, 4]");
}

#[test]
fn filter_on_infinite_range_peels_lazily() {
    let src = "match filter(x => x % 2 == 0, [0..])
  a :: b :: c :: _ -> [a, b, c]";
    assert_eq!(eval(src), "[0, 2, 4]");
}

#[test]
fn map_on_finite_range_flattens_for_display() {
    assert_eq!(eval("map(x => x * 2, [0..3])"), "[0, 2, 4]");
}

#[test]
fn cons_equals_eager_list() {
    // Equality flattens the cons spine, so a lazily built stream compares
    // equal to a fully evaluated list with the same elements.
    assert_eq!(eval("map(x => x * 2, [0..3]) == [0, 2, 4]"), "true");
    assert_eq!(eval("(1 :: [2, 3]) == [1, 2, 3]"), "true");
}
