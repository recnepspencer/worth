//! Content obeys Store's actual durable publication fate and schema admission.
use super::fixture::*;
use std::{num::NonZeroU64, path::Path};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    content::RelationalContentDenial, transactions::TransactionCommitError,
};
use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store_relational::{RelationalStoreHome, RelationalStoreHomeOpenRefusal};
use worth_testkit_relational::content_grid::ExactSampleGrid;

fn grid(first: i64) -> ExactSampleGrid {
    ExactSampleGrid {
        rows: 2,
        columns: 2,
        samples: vec![first, -1, 0, i64::MAX],
    }
}
fn faulted(
    path: &Path,
    execution: &ExecutionRequest<'_, '_>,
    faults: &[crate::media_faults::Fault],
) -> (
    RelationalStoreHome,
    Vec<worth_store::physical_runtime::certification::CertificationMediaFaultActivation>,
) {
    let media = crate::media_faults::FaultedMedia::admit(faults).unwrap();
    let home = RelationalStoreHome::certification_open_with_media(
        request(path, 1),
        media.admission,
        execution,
    )
    .unwrap();
    (home, media.activations)
}

#[test]
fn content_proven_no_effect_preserves_selected_bytes_and_reopen() {
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).unwrap();
    let (old, entity) = publish(&home, &execution, 1, None);
    let before = home
        .runtime()
        .observe_branch(&home.runtime().main_branch_identity())
        .unwrap()
        .0;
    home.certification_serving()
        .certification_begin_lifecycle_termination();
    let (new, result) = try_publish_grid(&home, &execution, grid(2), Some(entity));
    assert!(matches!(result, Err(TransactionCommitError::PublicationDenied {
        denial: worth_relational::facade::mvcc::RelationalPublicationDenial::DurablePublicationRefused(worth_relational::facade::durability::RelationalDurablePublicationRefusal::ProvenNoEffect { .. }), ..
    })), "{result:?}");
    assert!(result
        .as_ref()
        .unwrap_err()
        .durability_unresolved_fate()
        .is_none());
    assert_eq!(
        home.runtime()
            .observe_branch(&home.runtime().main_branch_identity())
            .unwrap()
            .0,
        before
    );
    assert_value(&home, &execution, entity, old, 1);
    let (_, basis) = home
        .runtime()
        .observe_branch(&home.runtime().main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.read_content(&execution, new, FORMAT).unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    drop(basis);
    home.certification_abort();
    let home = open(&location, &execution).unwrap();
    assert_value(&home, &execution, entity, old, 1);
    home.close();
}

#[test]
fn content_indeterminate_fate_fences_then_recovers_the_actual_durable_value() {
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).unwrap();
    let (_, entity) = publish(&home, &execution, 1, None);
    home.close();
    // Derive the final positioned-write coordinate from a genuine content commit.
    let (home, counters) = faulted(&location, &execution, &crate::media_faults::counters());
    crate::media_faults::arm(&counters);
    let (old, _) = publish(&home, &execution, 2, Some(entity));
    let writes = crate::media_faults::counts(&counters)
        .into_iter()
        .find(|(role, _)| *role == MediaOperationRole::PositionedWrite)
        .unwrap()
        .1;
    home.close();
    let (home, activations) = faulted(
        &location,
        &execution,
        &[(
            MediaOperationRole::PositionedWrite,
            NonZeroU64::new(writes).unwrap(),
            MediaFaultDirective::IndeterminateAfterEffect,
        )],
    );
    crate::media_faults::arm(&activations);
    let (new, result) = try_publish_grid(&home, &execution, grid(3), Some(entity));
    let fate = result
        .as_ref()
        .unwrap_err()
        .durability_unresolved_fate()
        .unwrap()
        .clone();
    assert_value(&home, &execution, entity, old, 2);
    let (_, basis) = home
        .runtime()
        .observe_branch(&home.runtime().main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.read_content(&execution, new, FORMAT).unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    drop(basis);
    let (_, blocked) = try_publish_grid(&home, &execution, grid(4), Some(entity));
    assert_eq!(
        blocked.as_ref().unwrap_err().durability_unresolved_fate(),
        Some(&fate)
    );
    home.certification_abort();
    let home = open(&location, &execution).unwrap();
    assert_value(&home, &execution, entity, new, 3);
    home.close();
}

#[test]
fn content_wrong_installed_version_refuses_without_erasing_the_home() {
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).unwrap();
    let (id, entity) = publish(&home, &execution, i64::MIN, None);
    home.checkpoint(&execution).unwrap();
    home.close();
    let refusal = match RelationalStoreHome::open(request(&location, 2), &execution) {
        Ok(_) => panic!("an incompatible installed format must refuse"),
        Err(refusal) => refusal,
    };
    assert!(
        matches!(
            refusal,
            RelationalStoreHomeOpenRefusal::Readmission(
                worth_relational::facade::durability::RelationalRecoveryStop::Content(
                    RelationalContentDenial::UnsupportedVersion
                )
            )
        ),
        "{refusal:?}"
    );
    let home = open(&location, &execution).unwrap();
    assert_value(&home, &execution, entity, id, i64::MIN);
    home.close();
}
