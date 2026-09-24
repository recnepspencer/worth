use std::cell::Cell;

use super::select_prior_preserved_or_created;
use crate::domain_computation::primary_graph::{
    WorthQueryPriorOutputDenial, WorthQueryPriorOutputDenialKind,
};

#[test]
fn only_absent_preservation_uses_initial_output() {
    let initial_called = Cell::new(false);
    let selected =
        select_prior_preserved_or_created(Ok::<_, WorthQueryPriorOutputDenial>(None), || {
            initial_called.set(true);
            Ok(7)
        });
    assert_eq!(selected.unwrap(), 7);
    assert!(initial_called.replace(false));

    let selected =
        select_prior_preserved_or_created(Ok::<_, WorthQueryPriorOutputDenial>(Some(11)), || {
            initial_called.set(true);
            Ok(7)
        });
    assert_eq!(selected.unwrap(), 11);
    assert!(!initial_called.get());

    let denied = WorthQueryPriorOutputDenial::new(
        WorthQueryPriorOutputDenialKind::OutputUnavailable,
        "geometry",
    );
    let result = select_prior_preserved_or_created(Err(denied.clone()), || {
        initial_called.set(true);
        Ok(7)
    });
    assert_eq!(result.unwrap_err(), denied);
    assert!(
        !initial_called.get(),
        "a denied preserved output cannot fall back"
    );
}
