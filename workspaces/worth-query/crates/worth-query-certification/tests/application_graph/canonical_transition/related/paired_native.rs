//! Actual pair reconstruction carries one request and publishes its image atomically.
use super::{derived_query, host, membership, output, paired_query, projection, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::{num::NonZeroUsize, sync::Arc};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::ApplicationScalarValueBinding},
    primary_graph::*,
};
#[derive(Debug, PartialEq)]
pub struct PairTotals {
    pub first: i64,
    pub second: i64,
}
impl WorthQueryManagedDerivedValue for PairTotals {
    fn retained_bytes(&self) -> usize {
        0
    }
}
pub struct Retained {
    pub view: WorthQueryManagedDerivedView<membership::MembershipQuery, PairTotals>,
    pub keys: Vec<WorthQueryManagedDerivedViewKey>,
    pub snapshot: WorthQueryManagedDerivedViewSnapshot<PairTotals>,
    pub owned: Arc<PairTotals>,
}
pub fn reconstruct(host: &host::Host, expected: i64, exercise_stops: bool) -> Retained {
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
            let members = schema
                .installed_query_binding::<membership::MembershipBinding>()
                .unwrap();
            let first = schema
                .installed_query_binding::<derived_query::ReadBinding>()
                .unwrap();
            let second = schema
                .installed_query_binding::<paired_query::ReadBinding>()
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    members.principal_binding(),
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let roots = ["derived-report", "derived-left", "derived-right"].map(|id| {
                selected
                    .resolve_entity(
                        &phase,
                        DerivedIdentityField::reference(),
                        id.to_owned(),
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
                WorthQueryProductQueryControls::new(NonZeroUsize::new(3).unwrap(), &scope)
            };
            let census = runtime
                .execute_application_query_one_shot(
                    &phase,
                    runtime
                        .on_branch(runtime.current_world())
                        .select(&phase)
                        .unwrap()
                        .admit_application_query(
                            &phase,
                            members.query(),
                            &access[0],
                            ApplicationQueryParameterSet::new(),
                            controls(),
                        )
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(census.rows()[0].rows.len(), 3);
            let view = runtime
                .open_managed_derived_collection_pair(
                    &ApplicationDerivedViewDefinition::new(
                        "related-codec-pair",
                        membership::MembershipQuery::reference(),
                        ApplicationDerivedViewLimits::bounded(3, 128 * 1024),
                    ),
                    members.query(),
                    first.query(),
                    second.query(),
                    selected.product(),
                )
                .unwrap();
            let run = |work, cancel_after_first| {
                projection::reset();
                output::reset_decodes();
                let cancellation = CancellationSource::new();
                let child = execution
                    .in_scope(|parent| {
                        let parent = parent.expect("actual installed host lease");
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
                let request = ExecutionRequest::leased(&child);
                let mut projected = 0;
                let (result, report) = request
                    .run(ExecutionWorkCeiling::new(Some(work)), |_| {
                        runtime.reconstruct_managed_derived_collection_pair(
                            request,
                            &view,
                            selected.product(),
                            &census,
                            |row| row.rows.clone(),
                            |identity: &String| {
                                let index = ["derived-report", "derived-left", "derived-right"]
                                    .iter()
                                    .position(|candidate| *candidate == identity.as_str())
                                    .unwrap();
                                let a = runtime
                                    .on_branch(runtime.current_world())
                                    .select(&phase)
                                    .unwrap()
                                    .admit_application_query(
                                        &phase,
                                        first.query(),
                                        &access[index],
                                        ApplicationQueryParameterSet::new(),
                                        controls(),
                                    )
                                    .unwrap();
                                let b = runtime
                                    .on_branch(runtime.current_world())
                                    .select(&phase)
                                    .unwrap()
                                    .admit_application_query(
                                        &phase,
                                        second.query(),
                                        &access[index],
                                        ApplicationQueryParameterSet::new(),
                                        controls(),
                                    )
                                    .unwrap();
                                Ok(WorthQueryDerivedPairReadPlans::new(a, b))
                            },
                            |value| output::literals(value).1,
                            |value| output::literals(value).1,
                            |a, b| {
                                projected += 1;
                                assert_eq!(output::literals(a).1, expected);
                                assert_eq!(output::literals(b).1, expected);
                                if cancel_after_first && projected == 1 {
                                    cancellation.cancel();
                                }
                                PairTotals {
                                    first: output::literals(a).1,
                                    second: output::literals(b).1,
                                }
                            },
                        )
                    })
                    .unwrap();
                (result, projected, report)
            };
            let (keys, projected, funded) = run(8_000_000, false);
            let keys = keys.unwrap();
            assert_eq!(projected, 3);
            assert_eq!(output::ignored_refusals(), 0);
            if exercise_stops {
                view.discard();
                let worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(wire),
                ) = output::DerivedBinding::encode(&output::calculate(&super::heavy_value()))
                    .unwrap()
                else {
                    panic!("real canonical carrier")
                };
                let narrow = funded
                    .charged_work()
                    .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap())
                    .unwrap();
                let (refused, projected, _) = run(narrow, false);
                let Err(WorthQueryManagedDerivedViewDenial::ReadDenied{denial,..})=refused else {panic!("late admitted pair decode must retain its precise native read denial: {refused:?}")};
                assert_eq!(denial.kind(),WorthQueryApplicationOneShotDenialKind::Projection(WorthQueryApplicationProjectionDenialKind::ValueDecode(WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::WorkCeiling))));
                assert!(projected > 0 && projected < 3);
                assert!(output::ignored_refusals() > 0);
                assert_eq!(projection::count(), 3);
                assert!(runtime
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&keys[0])
                    .is_err());
                let (cancelled, projected, _) = run(8_000_000, true);
                assert!(matches!(
                    cancelled,
                    Err(WorthQueryManagedDerivedViewDenial::Cancelled)
                ));
                assert_eq!(projected, 1);
                assert!(runtime
                    .observe_managed_derived_view(&view)
                    .unwrap()
                    .get(&keys[0])
                    .is_err());
                let (retry, projected, _) = run(8_000_000, false);
                assert_eq!(retry.unwrap(), keys);
                assert_eq!(projected, 3);
            }
            let snapshot = runtime.observe_managed_derived_view(&view).unwrap();
            for key in &keys {
                assert_eq!(
                    *snapshot.get(key).unwrap().unwrap(),
                    PairTotals {
                        first: expected,
                        second: expected
                    }
                );
            }
            let owned = snapshot.get(&keys[0]).unwrap().unwrap();
            Retained {
                view,
                keys,
                snapshot,
                owned,
            }
        })
        .unwrap()
}
