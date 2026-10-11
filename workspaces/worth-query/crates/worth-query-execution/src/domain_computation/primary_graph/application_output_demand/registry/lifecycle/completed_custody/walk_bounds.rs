//! Completed custody's stale-owner walk ignores a larger neighboring occurrence.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::{
    key, record,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use core::convert::Infallible;

impl DemandRegistryState {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn assert_completed_custody_walk<
        C,
    >(
        mut payer: impl FnMut() -> C,
    ) where
        C: OccurrenceSearchCharge<Refusal = Infallible>,
    {
        let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
        let commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        let (scope, _) = crate::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence();
        let mut contacts = Vec::new();
        for neighbors in [1, 64] {
            let selected = key("own", 1, 1);
            let mut state = DemandRegistryState::default();
            let mut row = record(
                receipt.product_branch().occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "",
                )),
                0,
            );
            row.source_commits.push(commit.clone());
            state.records.insert(selected.clone(), row);
            state.records.insert(
                key("own", 2, 1),
                record(
                    receipt.product_branch().occurrence(),
                    DemandState::Admitted,
                    0,
                ),
            );
            for generation in 1..=neighbors {
                state.records.insert(
                    key("larger-other", generation, 2),
                    record(
                        receipt.product_branch().occurrence(),
                        DemandState::Admitted,
                        0,
                    ),
                );
            }
            state.source_custody.insert(
                commit.clone(),
                SourceCustody {
                    occurrence: receipt.product_branch().occurrence(),
                    root_kind: PreparedOutputRootKind::Discovered(std::any::TypeId::of::<()>()),
                    source: None,
                    discovery: None,
                    bound_sources: Some(vec![BoundOutputSource {
                        scope,
                        identity: selected.source.clone(),
                    }]),
                    consumed_sources: vec![selected.source.clone()],
                    retired_sources: Vec::new(),
                    retired: None,
                    token_count: 0,
                    completed: true,
                },
            );
            let before = super::super::super::record_map::occurrence_rows_visited();
            match state.prune_completed_custody(&mut payer()) {
                Ok(()) => {}
                Err(never) => match never {},
            }
            assert!(state.source_custody.is_empty());
            assert!(state.records[&selected].source_commits.is_empty());
            contacts.push(super::super::super::record_map::occurrence_rows_visited() - before);
        }
        assert_eq!(contacts, vec![2, 2]);
    }
}
