//! Two real canonical pair members share decode custody until atomic publication.
#[path = "paired/first_query.rs"]
pub(super) mod first_query;
#[path = "paired/membership.rs"]
pub(super) mod membership;
use super::{host, opaque, repeated_query, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::facade::AspectValue;
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::declaration::application_schema::ApplicationScalarValueBinding;
use worth_query_host::facade::{declaration::application_query::*, primary_graph::*};

#[derive(Debug, PartialEq)]
struct PairRanks {
    first: i32,
    second: i32,
}
impl WorthQueryManagedDerivedValue for PairRanks {
    // Both ranks are inline; there is no retained heap payload.
    fn retained_bytes(&self) -> usize {
        0
    }
}

#[test]
fn late_swallowed_pair_decode_refusal_keeps_the_view_cold_and_fresh_retry_reconstructs() {
    let host = host::install(super::FUNDED_WORK);
    let runtime = host.runtime();
    let scope = request_scope();
    let external = authenticate_operator(host.installed_schema(), &scope);
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let schema = host.installed_schema();
            let members_binding = schema
                .installed_query_binding::<membership::MembershipBinding>()
                .unwrap();
            let first_binding = schema
                .installed_query_binding::<first_query::FirstBinding>()
                .unwrap();
            let second_binding = schema
                .installed_query_binding::<repeated_query::RepeatedBinding>()
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    members_binding.principal_binding(),
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let roots = ["root", "child"].map(|identity| {
                selected
                    .resolve_entity(
                        &phase,
                        DocumentIdentityField::reference(),
                        identity.to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            });
            let access = roots
                .iter()
                .map(|root| WorthQueryApplicationQueryAccessContext::new(&principal, root))
                .collect::<Vec<_>>();
            let controls = || {
                WorthQueryProductQueryControls::new(NonZeroUsize::new(2).unwrap(), &scope)
            };
            let membership = runtime
                .execute_application_query_one_shot(
                    &phase,
                    runtime
                        .on_branch(runtime.current_world())
                        .select(&phase)
                        .unwrap()
                        .admit_application_query(
                            &phase,
                            members_binding.query(),
                            &access[0],
                            ApplicationQueryParameterSet::new(),
                            controls(),
                        )
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(membership.rows()[0].rows.len(), 2);
            assert_eq!(
                membership.rows()[0]
                    .rows
                    .iter()
                    .map(|(_, identity)| identity.as_str())
                    .collect::<Vec<_>>(),
                ["root", "child"]
            );
            let view = runtime
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "canonical-paired-decode",
                        membership::MembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(2, 65_536),
                    ),
                    members_binding.query(),
                    first_binding.query(),
                    second_binding.query(),
                    selected.product(),
                )
                .unwrap();
            let run = |work, cancel_after_first| {
                repeated_query::reset();
                opaque::reset();
                let cancellation = CancellationSource::new();
                let lease = execution
                    .in_scope(|parent| {
                        let parent = parent.expect("the genuine host lends its execution lease");
                        parent
                            .child(LeaseRequest {
                                policy: ExecutionRequestPolicy::new(
                                    parent.policy().posture(),
                                    parent.policy().determinism(),
                                    ExecutionBudget::new(NonZeroUsize::MIN, Some(64 * 1024 * 1024), Some(work)),
                                ),
                                cancellation: cancellation.token(),
                                deadline: scope.deadline(),
                            })
                            .unwrap()
                    })
                    .unwrap();
                let request = ExecutionRequest::leased(&lease);
                let mut projected = 0;
                let (result, report) = request
                    .run(ExecutionWorkCeiling::new(Some(work)), |_| {
                        runtime.reconstruct_managed_derived_collection_pair(
                            request,
                            &view,
                            selected.product(),
                            &membership,
                            |row| row.rows.clone(),
                            |identity: &String| {
                                let index = ["root", "child"]
                                    .iter()
                                    .position(|candidate| *candidate == identity.as_str())
                                    .unwrap();
                                let first = runtime
                                    .on_branch(runtime.current_world())
                                    .select(&phase)
                                    .unwrap()
                                    .admit_application_query(
                                        &phase,
                                        first_binding.query(),
                                        &access[index],
                                        ApplicationQueryParameterSet::new(),
                                        controls(),
                                    )
                            .unwrap();
                        let second = runtime
                            .on_branch(runtime.current_world())
                            .select(&phase)
                            .unwrap()
                            .admit_application_query(
                                &phase,
                                second_binding.query(),
                                &access[index],
                                ApplicationQueryParameterSet::new(),
                                controls(),
                            )
                            .unwrap();
                        Ok(WorthQueryDerivedPairReadPlans::new(first, second))
                    },
                    |row| row.counter,
                    |row| row.counter,
                    |first, second| {
                        assert_eq!(first.value, super::value());
                        assert_eq!(second.value, super::value());
                        projected += 1;
                        if cancel_after_first && projected == 1 {
                            cancellation.cancel();
                        }
                        PairRanks {
                            first: first.value.children.iter().map(|child| child.rank).sum(),
                            second: second.value.children.iter().map(|child| child.rank).sum(),
                        }
                    },
                        )
                    })
                    .expect("the native scope ran the actual pair owner");
                (result, projected, report)
            };
            let (keys, projected, funded) = run(super::FUNDED_WORK, false);
            let keys = keys.expect("funded real pair reconstructs both members");
            assert_eq!(projected, 2);
            let snapshot = runtime.observe_managed_derived_view(&view).unwrap();
            for key in &keys {
                assert_eq!(
                    *snapshot.get(key).unwrap().unwrap(),
                    PairRanks {
                        first: 96,
                        second: 96
                    }
                );
            }
            view.discard();

            let AspectValue::String(worth_foundational::facade::InternedString::Raw(wire)) = super::binding::NestedBinding::encode(
                &opaque::SwallowingValue(super::value()),
            ).unwrap() else { panic!("the installed canonical binding has a string carrier"); };
            // Use the real native owner's public bill, not a copied budget
            // model. Its final canonical comparison charges exactly wire.len()
            // bytes. The last seed alone visits sixteen 256-byte tags, so
            // removing another half-wire reaches that seed after member one
            // was staged. The assertions below independently qualify that window.
            let narrow_work = funded.charged_work()
                .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap())
                .expect("the funded pair contains more work than its final decoder tail");
            let (refused, projected, _) = run(narrow_work, false);
            let refused = refused.expect_err("the narrow original pair request must refuse");
            let WorthQueryManagedDerivedViewDenial::ReadDenied { denial, .. } =
                refused
            else {
                panic!(
                    "late canonical decoding must preserve its native one-shot denial: {refused:?}; staged members={projected}; ignored seed refusals={}; completed swallowing projectors={}",
                    opaque::ignored_refusals(), repeated_query::returned(),
                );
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationOneShotDenialKind::Projection(
                    WorthQueryApplicationProjectionDenialKind::ValueDecode(
                        WorthQueryValueDecodeDenial::Checkpoint(
                            worth_execution::MapKernelStop::WorkCeiling
                        )
                    ),
                )
            );
            assert_eq!(
                projected, 1,
                "a complete earlier member was staged before the later refusal"
            );
            assert!(
                opaque::ignored_refusals() > 0,
                "the actual seed must swallow a native refusal; funded work={}, narrow work={narrow_work}, wire bytes={}",
                funded.charged_work(), wire.len(),
            );
            assert_eq!(
                repeated_query::returned(), 2,
                "the later real projector returned success after catching its field error"
            );
            assert!(matches!(
                runtime
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&keys[0]),
                Err(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
            ));

            // This separate caller controls only its original child request.
            // A successful first projector cannot publish a partial image once
            // its cancellation reaches the native owner stage.
            let (cancelled, projected, _) = run(super::FUNDED_WORK, true);
            assert!(matches!(
                cancelled,
                Err(WorthQueryManagedDerivedViewDenial::Cancelled)
            ), "the original child cancellation must retain its cause: {cancelled:?}");
            assert_eq!(projected, 1);
            assert!(matches!(
                runtime.observe_managed_derived_view(&view).unwrap().get(&keys[0]),
                Err(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
            ));

            let (retried, projected, _) = run(super::FUNDED_WORK, false);
            let retried = retried.expect("fresh funded request rebuilds the same cold view");
            assert_eq!(projected, 2);
            assert_eq!(retried, keys);
            let snapshot = runtime.observe_managed_derived_view(&view).unwrap();
            for key in &retried {
                assert_eq!(
                    *snapshot.get(key).unwrap().unwrap(),
                    PairRanks {
                        first: 96,
                        second: 96
                    }
                );
            }
        })
        .unwrap();
}
