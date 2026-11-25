use super::*;
use crate::{PlanarEdit, PlanarMutation};
use worth_query_consumer_values::{PlanarCurrentOutputExpectation, PlanarOperation};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

#[cfg(test)]
mod serial {
    use super::*;
    use worth_query_host::facade::runtime::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    const EXPECTATIONS: usize = 8_193;

    fn restore(
        contacts: &crate::producer::ProviderContacts,
        work: u64,
    ) -> support::DirApplication {
        let application = support::observed::open_directory(None, contacts);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        drop(settle(&request, &application));
        drop(principal);
        drop(scope);
        let directory = application.checkpoint_and_close_for_reopen();

        contacts.reset();
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(work)),
        );
        let limits = support::limits_with_policy(
            support::DEFAULT_RETAINED_COMMITS,
            16,
            64,
            support::invalidation(),
            policy,
        );
        support::observed::open_directory_with_limits(Some(directory), limits, contacts)
    }

    fn verification() -> PlanarEdit {
        PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".into(),
            // Each expectation resolves the entity, consumes current_output,
            // and reads the output field through the real decision reader.
            operation: PlanarOperation::VerifyCurrentOutputs(
                (0..EXPECTATIONS)
                    .map(|_| PlanarCurrentOutputExpectation {
                        producer_key: "anchor-a".into(),
                        output_key: "anchor-a".into(),
                    })
                    .collect(),
            ),
        })
    }

    #[test]
    #[expect(
        clippy::result_large_err,
        reason = "The fixture preserves the original typed owner denial."
    )]
    fn restored_current_output_is_tracked_before_any_producer_demand_and_rejects_source_change() {
        let _guard = checkpoint_recovery_test_guard();
        let provider_observation = crate::producer::ProviderContacts::default();
        // Fund all 8193 reads, including their record and lineage metadata.
        // The original finite 8M grant is tested separately below.
        let restored = restore(&provider_observation, 32_000_000);
        let (scope, principal) = authenticate(&restored);
        let request = restored.request(&principal, &scope);
        let verify = |command| {
            let observed = request
                .query(PlanarRead {
                    body_key: "anchor-a".into(),
                })
                .execute()
                .unwrap();
            request
                .advanced()
                .mutate(verification())
                .expect_source(observed.observed_sources()[0].clone())
                .idempotency(&command)
                .execute_in_program(
                    &restored,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
        };
        let outcome =
            verify(901_u64).expect("restored lineage is available to the ordinary tracked reader");
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "current output verification must commit"
        );
        assert_eq!(
            provider_observation.count(),
            0,
            "current_output must not warm a producer"
        );

        let (change_scope, change_principal) = authenticate(&restored);
        let change_request = restored.request(&change_principal, &change_scope);
        let observed = change_request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let changed = change_request
            .advanced()
            // The native action deliberately leaves the derived output stale.
            // Fixed source adjustment now publishes its required output too.
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "anchor-a".into(),
                operation: PlanarOperation::Adjust(vec![
                    worth_query_consumer_values::PlanarAdjustment {
                        body_key: "anchor-a".into(),
                        replacement_y: length(2),
                    },
                ]),
            }))
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&902_u64)
            .execute_in_program::<CheckpointProgram>(
                &restored,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert_eq!(
            provider_observation.count(),
            0,
            "native edit warms no producer"
        );
        let basis = || {
            restored
                .with_host_advancement(|phase| {
                    restored
                        .on_branch(restored.current_world())
                        .select(&phase)
                        .unwrap()
                        .describe_basis()
                })
                .unwrap()
        };
        let before_refusal = basis();
        let denial =
            verify(903_u64).expect_err("restored facts must still reject a changed source");
        use std::error::Error;
        use worth_query_host::facade::{
            application_entry::WorthQueryApplicationRequestMutationDenial,
            primary_graph::{
                MutationHandlerExecutionDenial, WorthQueryCurrentOutputDenial,
                WorthQueryCurrentOutputDenialKind,
            },
        };
        let WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Handler(handler),
        ) = denial
        else {
            panic!("{denial:?}")
        };
        let cause = handler
            .source()
            .and_then(|cause| cause.downcast_ref::<WorthQueryCurrentOutputDenial>())
            .expect("typed currentness cause");
        assert_eq!(cause.kind(), WorthQueryCurrentOutputDenialKind::StaleSource);
        assert_eq!(provider_observation.count(), 0);
        assert_eq!(
            basis(),
            before_refusal,
            "stale verification publishes no effects"
        );

        drop(settle(&request, &restored));
        assert!(provider_observation.count() > 0);
        let refreshed = request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .expect("the refreshed producer output is readable");
        assert_eq!(refreshed.rows()[0].value, length(3));
        assert!(matches!(
            verify(904_u64).expect("fresh output verifies after ordinary producer demand"),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }

    #[test]
    fn restored_current_output_original_request_grant_refuses_without_effects() {
        use worth_query_host::facade::{
            application_entry::WorthQueryApplicationRequestMutationDenial,
            primary_graph::{
                MutationHandlerExecutionDenial, RelationalExecutionDenialCause,
                WorthQueryInvariantProjectionDenialKind, WorthQueryOperationProjectionDenialKind,
            },
        };
        let _guard = checkpoint_recovery_test_guard();
        let contacts = crate::producer::ProviderContacts::default();
        let restored = restore(&contacts, 8_000_000);
        let basis = || {
            restored
                .with_host_advancement(|phase| {
                    restored
                        .on_branch(restored.current_world())
                        .select(&phase)
                        .unwrap()
                        .describe_basis()
                })
                .unwrap()
        };
        let before = basis();
        let (scope, principal) = authenticate(&restored);
        let request = restored.request(&principal, &scope);
        let observed = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let denial = request
            .advanced()
            .mutate(verification())
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&905_u64)
            .execute_in_program(
                &restored,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect_err("the original finite grant stops the actual repeated reader");
        let WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Projection(denial),
        ) = denial
        else {
            panic!("actual projection refusal: {denial:?}")
        };
        assert_eq!(
            denial.kind(),
            WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                    RelationalExecutionDenialCause::WorkExhausted
                )
            )
        );
        let work = denial
            .projection_work()
            .expect("actual reader work before refusal");
        assert!(work.field_reads() > 0 && work.field_reads() < EXPECTATIONS);
        assert_eq!(work.output_lineage_role_lookups(), work.field_reads());
        assert_eq!(basis(), before, "failed reads publish no native effects");
        assert_eq!(
            contacts.count(),
            0,
            "resource refusal never warms a producer"
        );
    }
}
