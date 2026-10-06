//! The currentness allowance decides reuse against a real native fact.

use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};
use worth_relational::facade::mvcc::CompanionPreflightBudget;

use super::{marked_facts_permit_reuse, InvalidationEditAdmission};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    tests::fixture::{installed_authorization_world, live_scope, AccountStatus},
    WorthQueryPrincipalResolutionMode,
};

fn currentness(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 1024 * 1024,
    })
}

#[test]
fn exhausted_currentness_while_reverifying_a_marked_fact_selects_fresh_without_stopping() {
    let world = installed_authorization_world(true);
    let entity = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        status.aspect().aspect_key().clone(),
        status.field_path().clone(),
    );

    world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| {
            let basis = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .unwrap();
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap();
            let revision = runtime
                .read_truth()
                .project_snapshot(&snapshot)
                .unwrap()
                .entity_field_revision(entity, &locator)
                .unwrap();
            let facts = [WorthQueryApplicationObservedFact::SourceFieldRevision {
                entity_id: entity,
                locator: locator.clone(),
                native_revision: Some(revision),
            }];

            // An ample allowance proves the marked fact current, so any `false`
            // below comes from exhaustion alone, never from a changed source.
            let mut ample = currentness(1 << 20);
            let full = ample.remaining_work();
            assert!(matches!(
                marked_facts_permit_reuse(&facts, runtime, &snapshot, &mut ample),
                Ok(true)
            ));
            let required = u64::try_from(full - ample.remaining_work()).unwrap();
            assert!(required > 1, "re-verification costs a visit plus its probe");

            // Every smaller allowance runs out inside re-verification. Each one
            // selects Fresh and leaves the demand running instead of returning
            // a terminal stop.
            for work in 1..required {
                assert!(
                    matches!(
                        marked_facts_permit_reuse(
                            &facts,
                            runtime,
                            &snapshot,
                            &mut currentness(work)
                        ),
                        Ok(false)
                    ),
                    "currentness work {work} of {required} must select Fresh"
                );
            }
            assert!(matches!(
                marked_facts_permit_reuse(&facts, runtime, &snapshot, &mut currentness(required)),
                Ok(true)
            ));
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        });
}
