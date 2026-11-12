//! Replacement preserves map ordering and the last assigned meaning.
use super::*;

#[test]
fn sorted_replacement_keeps_existing_keys_and_overrides_equal_key_once() {
    let original = NormalizedKeyCache::default()
        .replaced(vec![("z".into(), Symbol(1)), ("a".into(), Symbol(2))]);
    let replaced = original.replaced(vec![("m".into(), Symbol(3)), ("a".into(), Symbol(4))]);
    assert_eq!(replaced.get("a"), Some(&Symbol(4)));
    assert_eq!(replaced.get("m"), Some(&Symbol(3)));
    assert_eq!(replaced.get("z"), Some(&Symbol(1)));
    assert_eq!(replaced.get("missing"), None);
    assert_eq!(original.get("a"), Some(&Symbol(2)));
    assert!(replaced.rows.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert!(super::super::normalized_plan::slots::<(String, Symbol)>(usize::MAX).is_none());
}
