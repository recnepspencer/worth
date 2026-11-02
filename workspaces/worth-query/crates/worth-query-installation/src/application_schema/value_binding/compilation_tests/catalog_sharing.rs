//! Shared lookup must retain the installed entry without bypassing admission.
use super::*;
pub(super) fn check(
    catalogue: &WorthQueryInstalledApplicationValueBindingCatalog,
    retained: &WorthQueryInstalledApplicationValueBindingCatalog,
) {
    let binding = catalogue.field("Entity", "Aspect", "Field").unwrap();
    let mut admissions = 0;
    let mut prepare = |_, _| {
        admissions += 1;
        Ok::<_, ()>(())
    };
    let shared = catalogue
        .shared_field_admitted("Entity", "Aspect", "Field", &mut prepare)
        .unwrap()
        .unwrap();
    let again = retained
        .shared_field_admitted("Entity", "Aspect", "Field", &mut prepare)
        .unwrap()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&shared, &again));
    assert!(std::ptr::eq(binding, shared.as_ref()));
    assert_eq!(admissions, 2, "each shared lookup keeps actual admission");
    assert!(catalogue
        .shared_field_admitted("Entity", "Aspect", "Field", &mut |_, _| Err(
            "original refusal"
        ))
        .is_err());
}
