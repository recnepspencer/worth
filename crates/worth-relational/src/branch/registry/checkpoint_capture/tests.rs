//! Original registry DATA copies own their exact heaps independently of cells.
use super::*;
use crate::branch::{
    RelationalBranchReferenceCell, RelationalBranchRootDescriptor, RelationalBranchTarget,
};
use std::alloc::Layout;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_foundational::FoundationalBranchTarget;

fn source() -> RelationalBranchReferenceRegistry {
    let main = RelationalBranchReferenceCell::empty(7, BranchId("main".to_owned())).unwrap();
    main.advance_truth(FoundationalBranchTarget::basis(
        RelationalBranchTarget::new(
            7,
            11,
            13,
            vec![3, 5],
            RelationalBranchRootDescriptor::new([1; 32], [2; 32]),
        ),
    ))
    .unwrap();
    let fork = RelationalBranchReferenceCell::from_source(
        7,
        BranchId("fork".to_owned()),
        BranchId("main".to_owned()),
        &main.observation(),
    )
    .unwrap();
    let registry = RelationalBranchReferenceRegistry::from_main(main);
    registry.insert(fork);
    registry.retire_name(BranchId("deleted-original".to_owned()));
    registry
}

fn observation_bytes(value: &crate::branch::RelationalBranchReferenceObservation) -> usize {
    value.branch_id().as_str().len()
        + match value.target() {
            FoundationalBranchTarget::Empty => 0,
            FoundationalBranchTarget::Basis(target) => {
                Layout::array::<u64>(target.parent_commit_ids().len())
                    .unwrap()
                    .size()
            }
        }
}

fn data_bytes(value: &CheckpointBranchMetadata) -> u64 {
    let fields = Layout::new::<(usize, usize)>()
        .extend(Layout::new::<CheckpointBranchMetadataFields>())
        .unwrap()
        .0
        .pad_to_align()
        .size();
    let mut bytes = fields
        + value.runtime_name.len()
        + Layout::array::<RelationalBranchCellCheckpoint>(value.branch_cells.len())
            .unwrap()
            .size()
        + Layout::array::<DurableBranchLifecycleImage>(value.branch_lifecycle.len())
            .unwrap()
            .size()
        + Layout::array::<BranchId>(value.retired_branch_names.len())
            .unwrap()
            .size();
    for cell in value.branch_cells.iter() {
        bytes += cell.branch_id.0.len() + observation_bytes(&cell.observation);
        if let Some(value) = &cell.fork_provenance {
            bytes += observation_bytes(value);
        }
        if let Some(value) = &cell.fork_source_branch_id {
            bytes += value.0.len();
        }
    }
    bytes += value
        .branch_lifecycle
        .iter()
        .map(|value| value.branch_id.0.len())
        .sum::<usize>();
    bytes += value
        .retired_branch_names
        .iter()
        .map(|value| value.0.len())
        .sum::<usize>();
    u64::try_from(bytes).unwrap()
}

fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut observed = None;
    request
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}

#[test]
fn branch_metadata_clone_retains_exact_data_after_original_cells_die() {
    let registry = source();
    let expected = (
        registry.checkpoints(),
        registry.lifecycle_postures(),
        registry.retired_names(),
    );
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let captured = registry
        .capture_checkpoint_metadata(&ExecutionRequest::serial(&host), "original-runtime")
        .unwrap();
    assert_eq!(&*captured.branch_cells, expected.0.as_slice());
    assert_eq!(&*captured.branch_lifecycle, expected.1.as_slice());
    assert_eq!(&*captured.retired_branch_names, expected.2.as_slice());
    assert_eq!(
        captured.branch_cells[0]
            .observation
            .target()
            .as_basis()
            .unwrap()
            .parent_commit_ids(),
        &[3, 5]
    );
    let bill = data_bytes(&captured);
    assert_eq!(budget.charged_memory_bytes(), bill);
    let clone = captured.clone();
    assert!(std::ptr::eq(&*captured, &*clone));
    drop(captured);
    drop(registry);
    assert_eq!(clone.runtime_name, "original-runtime");
    assert_eq!(clone.branch_cells.len(), 2);
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(clone);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn branch_metadata_refuses_exact_data_bill_before_any_metadata_copy() {
    let registry = source();
    let ordinary = crate::test_execution_policy::serial_request();
    let baseline = registry
        .capture_checkpoint_metadata(&ExecutionRequest::serial(&ordinary), "runtime")
        .unwrap();
    let bill = data_bytes(&baseline);
    let scratch = u64::try_from(Layout::array::<CellSource>(2).unwrap().size()).unwrap();
    let budget = SerialMemoryBudget::new(Some(controller_memory() + scratch + bill - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let failure = registry
        .capture_checkpoint_metadata(&ExecutionRequest::serial(&host), "runtime")
        .unwrap_err();
    match failure {
        CheckpointBranchMetadataCaptureDenial::Request(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                requested,
                admitted,
                ancestor: 0,
            },
        ) => {
            assert_eq!(requested, bill);
            assert_eq!(admitted, bill - 1);
        }
        other => panic!("must enter the actual metadata admission: {other:?}"),
    }
    assert_eq!(budget.charged_memory_bytes(), 0);
    let after = registry
        .capture_checkpoint_metadata(&ExecutionRequest::serial(&ordinary), "runtime")
        .unwrap();
    assert_eq!(baseline, after);
}

#[test]
fn branch_metadata_stop_preserves_source_and_ordinary_projection_is_independent() {
    let registry = source();
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let stopped = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    assert!(matches!(
        registry.capture_checkpoint_metadata(&ExecutionRequest::serial(&stopped), "runtime"),
        Err(CheckpointBranchMetadataCaptureDenial::Request(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let original = registry
        .capture_checkpoint_metadata(&ExecutionRequest::serial(&host), "runtime")
        .unwrap();
    let projection = original.ordinary_owned_parts();
    assert_ne!(projection.0.as_ptr(), original.runtime_name.as_ptr());
    assert_ne!(projection.1.as_ptr(), original.branch_cells.as_ptr());
    drop(original);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(projection.0, "runtime");
    assert_eq!(projection.1, registry.checkpoints());
}
