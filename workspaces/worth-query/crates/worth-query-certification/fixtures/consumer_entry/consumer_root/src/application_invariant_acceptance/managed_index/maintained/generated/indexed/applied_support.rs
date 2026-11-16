//! The real indexed action acquires Source on D; final D still owns support and scope.
use super::super::super::{host, prefix, query, rule};
use super::super::source;
use super::{program, request};
use crate::application_invariant_acceptance::{
    authentication,
    managed_index::{lifecycle, prefix_observation},
};
use std::sync::Arc;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

#[test]
fn later_refinement_entrant_invalidates_applied_managed_source_before_atomic_publication() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix_observation::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls.clone())));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let current = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        // Seed a real A member that the enabled refinement excludes from the
        // earlier Parent. Each setup edit is an ordinary admitted Catalog action.
        use super::super::super::authoring::Edit;
        for (key, edit) in [
            Edit::Item {
                directory: "A".into(),
                identity: "future".into(),
                name: "sh-future".into(),
            },
            Edit::SetEnabled {
                directory: "A".into(),
                item: "future".into(),
                enabled: false,
            },
        ]
        .into_iter()
        .enumerate()
        {
            let result = current
                .advanced()
                .mutate(edit)
                .idempotency(&(0x919a_8610 + key as u64))
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ));
        }
        // The later stage has a real required-output root. Settle it now so the
        // unchanged Directory Source is reused, rather than creating an unrelated
        // membership change during the candidate under test.
        let baseline = current
            .advanced()
            .mutate(source::operation::RequestItem {
                directory: "A".into(),
                name: "ordinary-base".into(),
            })
            .without_source()
            .idempotency(&0x919a_8612_u64)
            .execute_performed::<host::Program, super::super::program::Root>(
                &world,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(baseline) = baseline else {
            panic!("the genuine ordinary required output primes before the candidate");
        };
        let raw_before = current
            .query(query::ReadSource {
                directory: "A".into(),
            })
            .execute()
            .unwrap();
        let future = raw_before
            .rows()
            .iter()
            .find(|row| row.identity == "future")
            .unwrap();
        assert_eq!((&*future.name, future.enabled), ("sh-future", false));
        let head = current
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let before = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "sh".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(before.rows().len(), 1);
        assert_eq!(
            (&*before.rows()[0].identity, &*before.rows()[0].name),
            ("x", "shared")
        );
        let index = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "sh".into(),
            })
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        let original = current
            .query(source::ReadDirectory {
                directory: "A".into(),
            })
            .execute()
            .unwrap();
        let mut envelope = current
            .edit_envelope()
            .idempotency(0x919a_8600_u64)
            .runtime_stages(&world)
            .ordered_authored()
            .unwrap();
        envelope
            .push_stage::<program::Root, _>(request::Request(source::operation::RequestItem {
                directory: "A".into(),
                name: "ordinary-base".into(),
            }))
            .unwrap();
        // Only the Item is authored here; the earlier stage owns Directory A.
        // This real SetEnabled operation makes an existing nonmatch enter the
        // earlier managed Parent on the same candidate.
        envelope
            .push_stage::<super::super::program::Root, _>(Edit::SetEnabled {
                directory: "A".into(),
                item: "future".into(),
                enabled: true,
            })
            .unwrap();
        let matches_before = calls
            .successful_matches
            .load(std::sync::atomic::Ordering::SeqCst);
        let outcome = envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation);
        let WorthQueryEditEnvelopeOutcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome.unwrap()
        else {
            panic!("a later Source entrant must refuse the complete candidate");
        };
        assert!(
            calls
                .successful_matches
                .load(std::sync::atomic::Ordering::SeqCst)
                > matches_before,
            "the real applied managed Parent was acquired during this candidate"
        );
        let failure = denial
            .invariant_execution_failure()
            .expect("complete applied Source support refusal");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &head
        );
        let after = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "sh".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            after.rows(),
            before.rows(),
            "neither entrant nor produced output is published"
        );
        let raw_after = current
            .query(query::ReadSource {
                directory: "A".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            raw_after.rows(),
            raw_before.rows(),
            "the real refinement edit is discarded"
        );
        assert!(
            raw_after
                .rows()
                .iter()
                .all(|row| row.identity != "managed-x-shared"),
            "the earlier producer's independently named output is not published"
        );
        assert_eq!(
            current
                .query(source::ReadDirectory {
                    directory: "A".into()
                })
                .execute()
                .unwrap()
                .rows(),
            original.rows()
        );
        drop(baseline);
        index.close();
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
        assert_eq!(after.rows(), before.rows(), "owned rows survive close");
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &head
        );
    });
}

#[test]
fn applied_managed_source_cannot_authorize_a_foreign_operation_scope() {
    lifecycle::run(|| {
        let calls = Arc::new(prefix_observation::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let current = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let head = current
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let a = current
            .query(query::FindPrefix {
                directory: "A".into(),
                prefix: "sh".into(),
            })
            .execute()
            .unwrap();
        let b = current
            .query(query::FindPrefix {
                directory: "B".into(),
                prefix: "sh".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(a.rows().len(), 1);
        assert_eq!(
            b.rows(),
            a.rows(),
            "the shared physical row cannot substitute for scope authority"
        );
        let original = current
            .query(source::ReadDirectory {
                directory: "B".into(),
            })
            .execute()
            .unwrap();
        let outcome = current
            .advanced()
            .mutate(request::Request(source::operation::RequestItem {
                directory: "B".into(),
                name: "sh-new".into(),
            }))
            .without_source()
            .idempotency(&0x919a_8601_u64)
            .execute_performed::<host::Program, program::Root>(
                &world,
                ExecutionAllocationPolicy::SystemAllocation,
            );
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
                denial,
            )),
        ) = outcome.unwrap()
        else {
            panic!("B Source cannot authorize the installed A operation");
        };
        assert_eq!(
            denial
                .coupled_producer_denial()
                .expect("actual required producer refusal")
                .kind(),
            WorthQueryOutputDemandDenialKind::ProducerUnavailable
        );
        assert_eq!(
            current.retain_current_read().unwrap().selected_commit(),
            &head
        );
        assert_eq!(
            current
                .query(source::ReadDirectory {
                    directory: "B".into()
                })
                .execute()
                .unwrap()
                .rows(),
            original.rows()
        );
        for directory in ["A", "B"] {
            assert!(current
                .query(query::FindPrefix {
                    directory: directory.into(),
                    prefix: "produced:".into()
                })
                .execute()
                .unwrap()
                .rows()
                .is_empty());
        }
    });
}

// The public connection contract permits multiple operations owned by the same
// feature to use its installed port. Only SetEnabled is needed by this court;
// admission, targeting and the handler remain the existing Catalog owners.
impl
    worth_query_decl::facade::application_program::WorthQueryApplicationRequiredOutputSource<
        super::super::super::schema::Catalog,
        super::super::program::Connection,
    > for super::super::super::authoring::Binding
{
    fn demand_from_source(
        input: &Self::Input,
    ) -> Result<
        super::super::producer::Demand,
        worth_query_decl::facade::application_program::WorthQueryRequiredOutputConnectionDenial,
    > {
        match input {
            super::super::super::authoring::Edit::SetEnabled { directory, .. } =>
                Ok(super::super::producer::Demand { directory: directory.clone() }),
            _ => Err(worth_query_decl::facade::application_program::WorthQueryRequiredOutputConnectionDenial::new(
                "the refinement court requires the actual item-enabled edit")),
        }
    }
}
