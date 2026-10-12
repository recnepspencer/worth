//! A selected publication joins only the binding that owns its delivery.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
};
use crate::domain_computation::primary_graph::tests::recoverable_commit_support;

#[test]
fn selected_delivery_keeps_both_producer_and_applicability_identity() {
    let receipt = recoverable_commit_support::committed_recoverable_application();
    let commit = receipt.committed_product_publication().composite_commit();
    let (scope, _) = source_custody::real_scope_and_occurrence();
    for other_producer in [true, false] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let own = key("z-own-binding", 1, 1);
        let requested = key("z-own-binding", 0, 1);
        let mut foreign = own.clone();
        if other_producer {
            foreign.producer = "a-other-binding".into();
        } else {
            foreign.applicability = WorthQueryProducerApplicability::new(
                "registry-fixture",
                WorthQueryProducerLifecyclePosture::Initial,
            );
        }
        assert!(foreign < own);
        assert!(requested < foreign);
        {
            let mut state = registry.state.lock().unwrap();
            for binding in [&foreign, &own] {
                let DemandState::Output(mut output) = closed_retirement::ready() else {
                    panic!("Ready fixture")
                };
                output.published_commit = Some(commit.clone());
                // Delivery can outlive an interrupted advancement. Semantic
                // joining must not hide the selected-publication fallback.
                output.finish_stopped(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                ));
                let mut row = record(occurrence(), DemandState::Output(output), 1);
                row.source_scope = Some(scope);
                state.records.insert(binding.clone(), row);
            }
        }
        let joined = registry
            .admit(
                requested,
                Some(super::super::SelectedOutputAdmission::new(commit, None)),
                scope,
                occurrence(),
                super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
                &mut record_admission(),
            )
            .unwrap();
        assert_eq!(joined.key, own, "other producer: {other_producer}");
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&own].interests, 2);
        assert_eq!(state.records[&foreign].interests, 1);
        assert_eq!(state.records.len(), 2);
        drop(state);
        drop(joined);
    }
}
