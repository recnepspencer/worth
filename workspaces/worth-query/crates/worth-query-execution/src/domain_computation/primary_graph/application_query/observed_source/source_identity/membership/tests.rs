use std::sync::Arc;

use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, VersionId};

use super::*;

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

fn footprint(endpoints: Vec<EntityId>, revision: u64) -> WorthQueryObservedSourceFootprint {
    let adjacency = WorthQueryObservedAdjacencyRevision {
        anchor: entity(1),
        relation_kind: KindId(7),
        direction: worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(revision)),
        comparison_work_limit: 8,
        endpoints,
    };
    WorthQueryObservedSourceFootprint {
        root: entity(1),
        complete: true,
        entities: vec![entity(1)],
        aspects: Vec::new(),
        root_selection: Some(Arc::new(WorthQueryObservedRootSelection::new(
            crate::domain_computation::primary_graph::source_facts::SourceFacts {
                entities: vec![entity(1)],
                aspects: Vec::new(),
                adjacencies: vec![adjacency.clone()],
                walks: None,
                derived: None,
                allocation: None,
            },
            None,
        ))),
        adjacencies: vec![adjacency],
    }
}

impl WorthQueryObservedSourceSelection {
    /// A complete retained selection for lineage tests outside this
    /// module, which cannot observe a source of their own.
    pub(in crate::domain_computation::primary_graph) fn selecting_for_test(endpoint: u64) -> Self {
        let authority = crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity::mint_for_test();
        let meaning = super::super::WorthQueryObservedSourceMeaningRegistry::new(authority)
            .intern(
                &[7; 32],
                &[8; 32],
                footprint(vec![entity(endpoint)], 4),
                &super::super::WorthQueryApplicationBasisSelectionIdentity::Relational,
            )
            .unwrap();
        Self {
            runtime_authority: authority.as_u64(),
            schema_binding: ApplicationSchemaBindingIdentity::from_installed_parts(
                1,
                2,
                CanonicalDigestId::new([3; 32]),
                CanonicalDigestId::new([4; 32]),
            ),
            meaning,
            managed: None,
        }
    }
}

#[test]
fn selected_membership_ignores_revisions_but_not_selected_endpoints() {
    let before = footprint(vec![entity(2)], 4);
    let same_selection = footprint(vec![entity(2)], 9);
    let changed_selection = footprint(vec![entity(3)], 9);
    assert!(same_footprint_membership(&before, &same_selection));
    assert!(!same_footprint_membership(&before, &changed_selection,));
}

#[test]
fn retained_selection_compares_a_fresh_meaning_without_revision_identity() {
    let authority = crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity::mint_for_test();
    let registry = super::super::WorthQueryObservedSourceMeaningRegistry::new(authority);
    let selection = super::super::WorthQueryApplicationBasisSelectionIdentity::Relational;
    let meaning = |endpoints, revision| {
        registry
            .intern(
                &[7; 32],
                &[8; 32],
                footprint(endpoints, revision),
                &selection,
            )
            .unwrap()
    };
    let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
        1,
        2,
        CanonicalDigestId::new([3; 32]),
        CanonicalDigestId::new([4; 32]),
    );
    let carrier = |meaning| WorthQueryObservedSourceSelection {
        runtime_authority: authority.as_u64(),
        schema_binding: binding.clone(),
        meaning,
        managed: None,
    };
    let before = carrier(meaning(vec![entity(2)], 4));
    let revision_only = carrier(meaning(vec![entity(2)], 9));
    let different = carrier(meaning(vec![entity(3)], 9));
    assert!(before
        .same_selected_membership_as(&revision_only, &mut |_| Ok(()))
        .unwrap());
    assert!(!before
        .same_selected_membership_as(&different, &mut |_| Ok(()))
        .unwrap());
}

#[test]
fn selection_comparison_requires_complete_membership() {
    let selected = footprint(vec![entity(2), entity(3)], 4);
    assert!(same_footprint_membership(&selected, &selected));
    let mut incomplete = selected.clone();
    incomplete.root_selection = None;
    assert!(!same_footprint_membership(&selected, &incomplete));
}

#[test]
fn text_comparison_reads_initialized_utf8_not_spare_capacity() {
    let mut spare = String::with_capacity(4096);
    spare.push_str("open");
    // Stale bytes past the initialized length stay in the spare capacity.
    let mut truncated = String::from("openXYZ");
    truncated.truncate("open".len());
    for left in [spare.as_str(), truncated.as_str()] {
        for right in ["open", "openXYZ", "opal", "closed", ""] {
            assert_eq!(same_text(left, right), left == right, "{left:?} {right:?}");
        }
    }
    assert!(same_text(&spare, &truncated));
}

#[test]
fn selected_membership_propagates_owned_admission_and_final_poll_stops() {
    let selected = WorthQueryObservedSourceSelection::selecting_for_test(1);
    let mut calls = 0;
    let stopped = selected.same_selected_membership_as(&selected, &mut |_| {
        calls += 1;
        Err(Cause::Cancelled)
    });
    assert_eq!(stopped, Err(Cause::Cancelled));
    assert_eq!(calls, 1);
    let last = selected.same_selected_membership_as(&selected, &mut |work| {
        if work == 0 {
            Err(Cause::DeadlineElapsed)
        } else {
            Ok(())
        }
    });
    assert_eq!(last, Err(Cause::DeadlineElapsed));
}
