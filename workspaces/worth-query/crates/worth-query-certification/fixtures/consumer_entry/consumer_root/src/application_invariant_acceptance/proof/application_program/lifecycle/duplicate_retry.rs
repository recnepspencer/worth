//! Same-key retry preserves the original receipt and schedules no new output.
use super::*;

pub(in super::super) fn duplicate_retry_does_not_schedule_again(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source = observed(&request, "anchor-b");
    let intent = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    let outcome = request
        .advanced()
        .mutate(intent.clone())
        .expect_source(source.clone())
        .idempotency(&10_008)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the first source operation must perform")
    };
    let original = performed.receipt().clone();
    let mut performed = performed
        .start_required_outputs(&world.application, &request, controls())
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()));
    let duplicate = request
        .advanced()
        .mutate(intent)
        .expect_source(source)
        .idempotency(&10_008)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    // D9 and the Phase4 receipt projection preserve recorded original meaning,
    // without reissuing live result/session evidence on an idempotent retry.
    let duplicate = match duplicate {
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => outcome,
        WorthQueryApplicationPerformedMutationOutcome::Performed(_) => {
            panic!("duplicate reexecuted")
        }
        WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { .. } => {
            panic!("duplicate required cleanup")
        }
        WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(_) => {
            panic!("published duplicate retained new custody")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(_) => {
            panic!("duplicate retained new output")
        }
        WorthQueryApplicationPerformedMutationOutcome::Blocked(_) => panic!("duplicate blocked"),
    };
    let worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::RecoveredCommitted(receipt) = duplicate else {
        panic!("identical key must recover its exact original receipt: {duplicate:?}");
    };
    assert!(original.matches_recorded_original(&receipt));
    assert_eq!(original.basis_descriptor(), receipt.basis_descriptor());
    assert_eq!(
        original.changed_record_count(),
        receipt.changed_record_count()
    );
    let settled = settle(&world.application, &mut performed, &request);
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(4)
    );
}
