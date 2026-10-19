//! A fact the rebase cannot pay to observe again is not decided, and is
//! never kept as its handler read it.

use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use super::tests::{account_note, planned, rebase_at_current, rebase_result_within};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, live_scope, AccountIdentity, AccountStatus,
            AuthorizationWorld, TouchAccountOperation,
        },
    },
};
use worth_execution::ExecutionAllocationPolicy;

#[test]
fn an_indexed_selection_the_width_cannot_pay_for_is_not_kept() {
    let world = installed_authorization_world(true);
    let selection = world
        .application
        .with_host_advancement(|phase| indexed_selection(&phase, &world))
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(
                matches!(
                    rebase_within_width(
                        &phase,
                        &host_custody.borrowed_request_for_test(),
                        &world,
                        selection.clone(),
                        false,
                        2
                    ),
                    RebasedSourceFacts::Exact(_)
                ),
                "the lookup and one candidate observe the empty selection again"
            )
        })
        .expect("the fixture read owns its installed host call");
    // The observation reads only what it reserved. One unit pays the lookup
    // and no candidate, so even an empty selection is not observed: a
    // lookup that may read nothing cannot tell empty from cut short.
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            for indexed_width in [0, 1] {
                for producer_output in [false, true] {
                    assert_eq!(
                        rebase_within_width(
                            &phase,
                            &host_custody.borrowed_request_for_test(),
                            &world,
                            selection.clone(),
                            producer_output,
                            indexed_width
                        ),
                        RebasedSourceFacts::VerificationRequired {
                            reason: RebaseVerificationReason::AdmissionDenied(
                                CompanionPreflightStop::WorkExhaustedWithoutRequirement {
                                    maximum: u64::try_from(indexed_width).unwrap(),
                                }
                            ),
                            own_effect: OwnEffectOnReads::NONE_ASKED,
                        },
                        "a budget miss is the request's stop, for a producer or not"
                    );
                }
            }
        })
        .expect("the fixture read owns its installed host call")
}

/// A source read's comparison is granted the most it can cost, so only one
/// that cannot answer leaves the commit's own effect undecidable.
#[test]
fn only_a_comparison_that_cannot_answer_is_undecidable() {
    let world = installed_authorization_world(true);
    let (entity, locator) = world
        .application
        .with_host_advancement(|phase| account_note(&phase, &world, "account-1"))
        .expect("the fixture read owns its installed host call");
    let kind = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .entity_kind(AccountIdentity::reference().entity())
        .unwrap();
    let read = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            rebase_at_current(
                &phase,
                &host_custody.borrowed_request_for_test(),
                &world,
                vec![WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: entity,
                    kind,
                    locator: planned(&locator),
                }],
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationObservedFact::SourceFieldRevision {
        locator: read_locator,
        native_revision,
        ..
    } = read[0].clone()
    else {
        panic!("a rebased field read is a source field revision: {read:?}");
    };
    let mut gone = entity;
    gone.generation.0 += 1;
    let gone_read = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: gone,
        locator: read_locator,
        native_revision,
    };
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                rebase_result_within(
                    &phase,
                    &phase.borrowed_request_for_test(),
                    &world,
                    vec![gone_read.clone()],
                    64
                ),
                RebasedSourceFacts::SupersededByOwnEffect {
                    facts: vec![gone_read].into(),
                    ordinals: Arc::from([0]),
                },
                "a field read whose entity is gone answers moved once its liveness check is paid"
            )
        })
        .expect("the fixture read owns its installed host call");
    let uncomparable = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            locator.aspect().aspect_key().clone(),
            CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
        ),
        native_revision,
    };
    let RebasedSourceFacts::VerificationRequired { own_effect, .. } = world
        .application
        .with_host_advancement(|phase| {
            rebase_result_within(
                &phase,
                &phase.borrowed_request_for_test(),
                &world,
                vec![uncomparable],
                64,
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a field revision of a live entity's undeclared field cannot be compared");
    };
    assert_eq!(own_effect, OwnEffectOnReads(OwnEffect::Undecidable));
    assert_eq!(
        own_effect.at_own_publication(true),
        FactlessCurrentness::Undecidable,
        "a comparison that cannot answer is denied, not refreshed"
    );
}

/// The retained selection a decision made for a value no account holds.
fn indexed_selection(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
) -> WorthQueryApplicationObservedFact {
    let request = live_scope();
    let actor = authenticated_principal(&(phase_custody).test_host_request(), world, &request);
    let account = resolved_account(
        &(phase_custody).test_host_request(),
        world,
        "open",
        &request,
    );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request())
        .authorize_operation(
            phase_custody,
            &actor,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, _| {
                assert!(reader
                    .decision_select_entities(AccountStatus::reference(), "missing".to_owned(), 2)
                    .unwrap()
                    .is_empty());
            },
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(phase_custody, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
        .decision_facts()
        .iter()
        .find(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
            )
        })
        .cloned()
        .expect("the decision retains its indexed selection")
}

fn rebase_within_width(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    selection: WorthQueryApplicationObservedFact,
    producer_output: bool,
    indexed_width: usize,
) -> RebasedSourceFacts {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(64),
        maximum_preparation_bytes: 1024 * 1024,
    });
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(execution, |runtime| {
            rebase(
                execution,
                runtime,
                selected.application_basis().snapshot_handle(),
                PreparedSourceFactRebase::admit(
                    vec![selection],
                    [].into(),
                    StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None),
                )
                .unwrap(),
                &BTreeSet::new(),
                producer_output,
                Some(indexed_width),
                &mut admission,
            )
        })
}
