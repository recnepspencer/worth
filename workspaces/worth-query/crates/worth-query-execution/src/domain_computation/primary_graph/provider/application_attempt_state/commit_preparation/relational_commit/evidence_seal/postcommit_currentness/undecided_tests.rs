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
    let selection = indexed_selection(&world);
    assert!(
        matches!(
            rebase_within_width(&world, selection.clone(), false, 2),
            RebasedSourceFacts::Exact(_)
        ),
        "the lookup and one candidate observe the empty selection again"
    );
    // The observation reads only what it reserved. One unit pays the lookup
    // and no candidate, so even an empty selection is not observed: a
    // lookup that may read nothing cannot tell empty from cut short.
    for (indexed_width, required) in [(0, 1), (1, 2)] {
        for producer_output in [false, true] {
            assert_eq!(
                rebase_within_width(&world, selection.clone(), producer_output, indexed_width),
                RebasedSourceFacts::VerificationRequired {
                    reason: RebaseVerificationReason::AdmissionDenied(
                        CompanionPreflightStop::WorkExhausted {
                            required,
                            maximum: u64::try_from(indexed_width).unwrap(),
                        }
                    ),
                    own_effect: OwnEffectOnReads::NONE_ASKED,
                },
                "a budget miss is the request's stop, for a producer or not"
            );
        }
    }
}

/// A source read's comparison is granted the most it can cost, so only one
/// that cannot answer leaves the commit's own effect undecidable.
#[test]
fn only_a_comparison_that_cannot_answer_is_undecidable() {
    let world = installed_authorization_world(true);
    let (entity, locator) = account_note(&world, "account-1");
    let kind = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .entity_kind(AccountIdentity::reference().entity())
        .unwrap();
    let read = rebase_at_current(
        &world,
        vec![WorthQueryApplicationObservedFact::AbsentField {
            entity_id: entity,
            kind,
            locator: planned(&locator),
        }],
    );
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
    assert_eq!(
        rebase_result_within(&world, vec![gone_read.clone()], 64),
        RebasedSourceFacts::SupersededByOwnEffect {
            facts: Arc::from([gone_read]),
            ordinals: Arc::from([0]),
        },
        "a field read whose entity is gone answers moved once its liveness check is paid"
    );
    let uncomparable = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            locator.aspect().aspect_key().clone(),
            CanonicalFieldPath::single(FieldKey::new("undeclared").unwrap()),
        ),
        native_revision,
    };
    let RebasedSourceFacts::VerificationRequired { own_effect, .. } =
        rebase_result_within(&world, vec![uncomparable], 64)
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
fn indexed_selection(world: &AuthorizationWorld) -> WorthQueryApplicationObservedFact {
    let request = live_scope();
    let actor = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&actor, &account, &operation, Default::default(), &request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
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
            admission,
            projection,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(ExecutionAllocationPolicy::SystemAllocation)
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
    world: &AuthorizationWorld,
    selection: WorthQueryApplicationObservedFact,
    producer_output: bool,
    indexed_width: usize,
) -> RebasedSourceFacts {
    let selected = world.selected_product();
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 64,
        maximum_preparation_bytes: 1024 * 1024,
    });
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(|runtime| {
            rebase(
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
                indexed_width,
                &mut admission,
            )
        })
}
