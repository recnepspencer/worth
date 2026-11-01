//! Original symbol input borrows exact admitted identity and cannot mint names.
use super::*;
use crate::symbols::data::StringInterner;
#[test]
fn original_symbols_keep_ids_require_recorded_references_and_never_reserve() {
    let mut original = StringInterner::default();
    let field = original.intern("field").unwrap();
    let other = original.intern("other").unwrap();
    let before = original.clone();
    let dictionary = SymbolTableSnapshot {
        entries: vec![(field, "field".into()), (other, "other".into())],
    };
    let references = [field];
    let mut input = OriginalSymbolMeanings::new(&original, &dictionary, &references).unwrap();
    assert_eq!(input.reserve_revision_reference("field").unwrap(), field);
    assert!(matches!(
        input.reserve_revision_reference("other"),
        Err(SymbolReservationDenial::InvalidDictionary)
    ));
    assert!(matches!(
        input.reserve_revision_reference("fresh"),
        Err(SymbolReservationDenial::InvalidDictionary)
    ));
    assert_eq!(original, before);
    let conflicting = SymbolTableSnapshot {
        entries: vec![(field, "other".into())],
    };
    assert!(matches!(
        OriginalSymbolMeanings::new(&original, &conflicting, &references),
        Err(SymbolReservationDenial::InvalidDictionary)
    ));
}
