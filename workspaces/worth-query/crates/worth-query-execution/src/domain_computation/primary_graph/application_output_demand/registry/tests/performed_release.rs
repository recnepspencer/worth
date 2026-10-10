//! Observation-pressure release follows declared commit order, including work.

use super::super::{PreparedOutputRootKind, SourceCustody, WorthQueryPerformedOutputDemandSource};
use super::*;
use crate::domain_computation::primary_graph::tests::{fixture, recoverable_commit_support};

#[test]
fn unheld_performed_sources_release_the_oldest_with_commit_order_work() {
    let world = fixture::installed_authorization_world(true);
    let mut sources = Vec::new();
    let mut current = "open".to_owned();
    for index in 0..8 {
        let replacement = format!("release-source-{index}");
        let mut receipt =
            recoverable_commit_support::commit_on_world_with_output_demand_observation(
                &world,
                170 + index,
                &current,
                &replacement,
            );
        let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation()
            .unwrap();
        sources.push(WorthQueryPerformedOutputDemandSource {
            receipt,
            change,
            observation,
            output_source_identity: None,
        });
        current = replacement;
    }
    let oldest = sources[0].change.product_commit().clone();
    let occurrence = sources[0].receipt.product_branch().occurrence();
    assert!(sources
        .windows(2)
        .all(|pair| pair[0].change.product_commit() < pair[1].change.product_commit()));
    let mut state = DemandRegistryState::default();
    for index in [7, 3, 5, 1, 6, 2, 4, 0] {
        let source = sources[index].clone();
        state.source_custody.insert(
            source.change.product_commit().clone(),
            SourceCustody {
                occurrence,
                root_kind: PreparedOutputRootKind::Required(std::any::TypeId::of::<()>()),
                source: Some(source),
                discovery: None,
                bound_sources: None,
                consumed_sources: Vec::new(),
                retired_sources: Vec::new(),
                retired: None,
                token_count: 0,
                completed: false,
            },
        );
    }
    for slot in 1..=3 {
        let mut row = record(occurrence, DemandState::Admitted, 0);
        row.source_commits.push(oldest.clone());
        state.records.insert(key("release-proof", 1, slot), row);
    }
    let mut admission = record_admission();
    let released = state
        .release_unheld_performed_source(&mut admission)
        .unwrap();
    assert!(released.is_some());
    assert!(!state.source_custody.contains_key(&oldest));
    assert_eq!(state.source_custody.len(), 7);
    assert!(state
        .records
        .values()
        .all(|row| !row.source_commits.contains(&oldest)));
    let charged = admission.charged_work();
    eprintln!("release-proof-count={charged}");
    // Eight source visits, one candidate's three row comparisons, and three
    // row visits releasing that candidate; later commits cannot replace it.
    assert_eq!(
        charged, 14,
        "release work must follow declared commit order"
    );
}
