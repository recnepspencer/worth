use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget as Budget, DerivedIndexMaintenanceDenialKind as Denial,
};
use worth_execution::ExecutionWorkCeiling;

fn cold_budget() -> Budget {
    Budget {
        maximum_work_units: 100_000,
        maximum_cold_record_slots: 100_000,
        maximum_derived_rows: 100_000,
    }
}

#[test]
fn cold_reconstruction_is_paid_and_short_requests_publish_nothing() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(execution, &runtime, "paid");
    let definition = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "paid.name".into(),
        branch_scoped: true,
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
    });
    let request = DerivedIndexBuildRequest {
        source_commit_id: created.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![definition.index_id],
    };
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    // A native independent build gives the required receipt before request funding.
    let baseline = runtime
        .index_authority()
        .refresh_for_basis(request.clone(), &basis, None, cold_budget(), execution)
        .unwrap();
    runtime
        .index_authority()
        .discard_generations(crate::indexes::data::DerivedIndexDiscardRequest::all_bases(
            definition.index_id,
        ))
        .unwrap();
    let needed = baseline.work.work_units as u64;
    let (short, _) = execution
        .run(ExecutionWorkCeiling::new(needed - 1), |_| {
            runtime.index_authority().refresh_for_basis(
                request.clone(),
                &basis,
                None,
                cold_budget(),
                execution,
            )
        })
        .unwrap();
    assert_eq!(
        short.unwrap_err().kind,
        Denial::Execution(Cause::WorkExhausted)
    );
    assert!(runtime
        .index_access()
        .latest_generation(definition.index_id, &BranchId("main".into()))
        .is_none());
    let next_host = crate::test_execution_policy::serial_request();
    let next_request = &worth_execution::ExecutionRequest::serial(&next_host);
    let (enough, report) = next_request
        .run(ExecutionWorkCeiling::new(needed), |_| {
            runtime.index_authority().refresh_for_basis(
                request,
                &basis,
                None,
                cold_budget(),
                next_request,
            )
        })
        .unwrap();
    let enough = enough.unwrap();
    assert_eq!(
        enough.generations[0].entries,
        baseline.generations[0].entries
    );
    assert_eq!(report.charged_work(), enough.work.work_units as u64);
    release_test_commit_snapshot(&runtime, &created);
}

#[test]
fn cold_retry_and_cancellation_use_the_same_request_payer() {
    let source = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(64 * 1024 * 1024),
        source.token(),
        None,
    );
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(execution, &runtime, "retry");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "retry.name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .index_id;
    let request = DerivedIndexBuildRequest {
        source_commit_id: created.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index],
    };
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let baseline = runtime
        .index_authority()
        .refresh_for_basis(request.clone(), &basis, None, cold_budget(), execution)
        .unwrap();
    runtime
        .index_authority()
        .discard_generations(crate::indexes::data::DerivedIndexDiscardRequest::all_bases(
            index,
        ))
        .unwrap();
    let mut ordinary = cold_budget();
    ordinary.maximum_cold_record_slots = 0;
    let first = runtime
        .index_authority()
        .refresh_for_basis(request.clone(), &basis, None, ordinary, execution)
        .unwrap_err();
    assert_eq!(first.kind, Denial::ColdReconstructionRequired);
    let needed = (first.work.work_units + baseline.work.work_units) as u64;
    let (retry, _) = execution
        .run(ExecutionWorkCeiling::new(needed - 1), |_| {
            let first = runtime
                .index_authority()
                .refresh_for_basis(request.clone(), &basis, None, ordinary, execution)
                .unwrap_err();
            assert_eq!(first.kind, Denial::ColdReconstructionRequired);
            runtime.index_authority().refresh_for_basis(
                request.clone(),
                &basis,
                None,
                cold_budget(),
                execution,
            )
        })
        .unwrap();
    let retry_kind = retry
        .map(|_| None)
        .unwrap_or_else(|denied| Some(denied.kind));
    runtime
        .index_authority()
        .discard_generations(crate::indexes::data::DerivedIndexDiscardRequest::all_bases(
            index,
        ))
        .unwrap();
    let mut charges = 0;
    let rendezvous = crate::indexes::at_next_charges(move || {
        charges += 1;
        if charges == 2 {
            source.cancel();
        }
    });
    let cancelled = runtime.index_authority().refresh_for_basis(
        request,
        &basis,
        None,
        cold_budget(),
        execution,
    );
    drop(rendezvous);
    let cancelled_kind = cancelled
        .map(|_| None)
        .unwrap_or_else(|denied| Some(denied.kind));
    assert_eq!(
        (retry_kind, cancelled_kind),
        (
            Some(Denial::Execution(Cause::WorkExhausted)),
            Some(Denial::Execution(Cause::Cancelled))
        )
    );
    assert!(runtime
        .index_access()
        .latest_generation(index, &BranchId("main".into()))
        .is_none());
    release_test_commit_snapshot(&runtime, &created);
}

mod seeded_shapes;
