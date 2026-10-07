use worth_store_physical_format::PhysicalPageSizeClass;

use super::super::validate_blob_semantic;
use super::*;
use crate::redo_replay::plan::supersession::admit_scratch_bytes;
use crate::redo_replay::terminal_head_retirement_fixture::{
    format, projection, record, selected_terminal_head, selected_terminal_head_in, store, store_of,
    survivor, terminal_head,
};

const INVALID: Result<(), PhysicalRedoPlanningDenial> =
    Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);

#[test]
fn a_terminal_head_retirement_is_bound_to_this_store_and_recomputed_in_its_format() {
    let selected = selected_terminal_head(vec![terminal_head(), survivor()]);
    let claimed = projection(selected.retirement);
    assert_eq!(
        validate_release_head_effect(&[], &claimed, store(), format()),
        Ok(())
    );
    assert_eq!(
        validate_release_head_effect(&[], &claimed, store_of([8; 16]), format()),
        INVALID,
        "the source basis names another store"
    );

    let wide = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let foreign = selected_terminal_head_in(vec![terminal_head(), survivor()], wide);
    let foreign = projection(foreign.retirement);
    assert_eq!(
        validate_release_head_effect(&[], &foreign, store(), wide),
        Ok(())
    );
    assert_eq!(
        validate_release_head_effect(&[], &foreign, store(), format()),
        INVALID,
        "the removal does not recompute from the carried path in this format"
    );
}

#[test]
fn a_terminal_head_retirement_is_charged_as_a_head_tree_claim() {
    let selected = selected_terminal_head(vec![terminal_head(), survivor()]);
    let framed = selected.retirement.framed_bytes().unwrap();
    let entries = selected.retirement.entry_count().unwrap();
    let claimed = projection(selected.retirement);
    let charged = admit_scratch_bytes(0, &[], &claimed, u64::MAX).unwrap();
    assert_eq!(charged, framed * 3 + entries * 4096);
    assert_eq!(
        admit_scratch_bytes(0, &[], &claimed, charged - 1),
        Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: charged,
            admitted: charged - 1,
        })
    );
}

#[test]
fn no_data_record_validates_under_a_terminal_head_retirement() {
    let selected = selected_terminal_head(vec![terminal_head()]);
    let claimed = projection(selected.retirement);
    assert_eq!(
        validate_blob_semantic(b"redo-record", record(9), store().bytes(), &claimed),
        INVALID
    );
}
