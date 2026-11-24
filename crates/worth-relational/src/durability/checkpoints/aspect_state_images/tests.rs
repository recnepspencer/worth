use super::{export_state, CheckpointAspectContractCatalog};
use crate::storage::substrate::assert_unique_extra_move_funding;

#[test]
fn unique_extra_move_and_borrowed_export_preserve_original_funding() {
    assert_unique_extra_move_funding(|state, contract| {
        let catalog = CheckpointAspectContractCatalog::from_contracts(&[contract]).unwrap();
        assert!(export_state(state, Some(&catalog)).unwrap().is_some());
    });
}
