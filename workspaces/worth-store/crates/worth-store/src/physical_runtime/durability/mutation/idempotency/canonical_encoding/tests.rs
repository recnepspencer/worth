use super::*;

fn encode(target: &mut impl CanonicalBindingEncoding) {
    target.field(&[3, 5]);
    target.push(7);
    target.write(&[11, 13]);
}

#[test]
fn one_field_grammar_produces_fixed_bytes_and_compares_without_heap_storage() {
    let expected = [2, 0, 0, 0, 0, 0, 0, 0, 3, 5, 7, 11, 13];
    let mut produced = Vec::new();
    encode(&mut produced);
    assert_eq!(produced, expected);
    let mut comparison = CanonicalBindingComparison::new(&expected);
    encode(&mut comparison);
    assert!(comparison.matches());
}

#[test]
fn comparison_rejects_altered_short_and_trailing_input() {
    let expected = [2, 0, 0, 0, 0, 0, 0, 0, 3, 5, 7, 11, 13];
    let mut altered = expected;
    altered[9] = 6;
    for bytes in [&altered[..], &expected[..12], &expected[..0]] {
        let mut comparison = CanonicalBindingComparison::new(bytes);
        encode(&mut comparison);
        assert!(!comparison.matches());
    }
    let trailing = [2, 0, 0, 0, 0, 0, 0, 0, 3, 5, 7, 11, 13, 17];
    let mut comparison = CanonicalBindingComparison::new(&trailing);
    encode(&mut comparison);
    assert!(!comparison.matches());
}

#[test]
fn empty_encoding_requires_empty_input() {
    assert!(CanonicalBindingComparison::new(&[]).matches());
    assert!(!CanonicalBindingComparison::new(&[0]).matches());
}
