//! Genuine scalar directory construction across page growth and AVL rotations.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn controlled_scalar_directory_preserves_bijection_and_final_owner_refund() {
    let budget = SerialMemoryBudget::new(Some(2 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut directory = RecordSlotDirectory::default();
    for index in 0..129 {
        directory
            .insert_for_request(1000 + index * 17, &request)
            .unwrap();
    }
    let old = directory.clone();
    directory.insert_for_request(9, &request).unwrap();
    for physical in 0..129 {
        let logical = 1000 + physical * 17;
        assert_eq!(directory.physical_index(logical), Some(physical));
        assert_eq!(directory.logical_slot(physical), Some(logical));
        assert_eq!(old.physical_index(logical), Some(physical));
    }
    assert_eq!(directory.physical_index(9), Some(129));
    assert_eq!(old.physical_index(9), None);
    assert!(budget.charged_memory_bytes() > 0);
    drop(directory);
    assert!(budget.charged_memory_bytes() > 0);
    drop(old);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
