//! Binding without an admission must not read pending completed custody as truth.
use super::super::{BoundOutputSource, PreparedOutputRootKind, SourceCustody};
use super::*;
use crate::domain_computation::primary_graph::tests::{fixture, recoverable_commit_support};
use crate::equality_type_id::EqualityTypeId;

#[test]
fn registry_binding_reader_resolves_completed_custody_note() {
    let world = fixture::installed_authorization_world(true);
    let mut active = recoverable_commit_support::commit_on_world_with_output_demand_observation(
        &world,
        201,
        "open",
        "binding-active",
    );
    let change = Arc::new(active.take_performed_relational_product_change().unwrap());
    let observation = active
        .committed_product_publication()
        .take_output_demand_observation()
        .unwrap();
    let source = super::super::WorthQueryPerformedOutputDemandSource {
        receipt: active,
        change,
        observation,
        output_source_identity: None,
    };
    let completed = recoverable_commit_support::commit_on_world_with_output_demand_observation(
        &world,
        202,
        "binding-active",
        "binding-completed",
    );
    let current_commit = source.change.product_commit().clone();
    let completed_commit = completed
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = source.receipt.product_branch().occurrence();
    let (scope, _) = source_custody::real_scope_and_occurrence();
    for seed in 0..16 {
        let selected = BoundOutputSource {
            scope,
            identity: key("binding", seed + 1, 3).source,
        };
        let newer = BoundOutputSource {
            scope,
            identity: key("binding", seed + 2, 3).source,
        };
        let make = || {
            let registry = WorthQueryOutputDemandRegistry::default();
            let mut state = registry.state.lock().unwrap();
            state.source_custody.insert(
                current_commit.clone(),
                SourceCustody {
                    occurrence,
                    root_kind: PreparedOutputRootKind::Discovered(EqualityTypeId::of::<()>()),
                    source: Some(source.clone()),
                    discovery: None,
                    bound_sources: None,
                    consumed_sources: Vec::new(),
                    retired_sources: Vec::new(),
                    retired: None,
                    token_count: 1,
                    cleanup: Default::default(),
                    completed: false,
                },
            );
            state.source_custody.insert(
                completed_commit.clone(),
                SourceCustody {
                    occurrence,
                    root_kind: PreparedOutputRootKind::Discovered(EqualityTypeId::of::<()>()),
                    source: None,
                    discovery: None,
                    bound_sources: Some(vec![newer.clone()]),
                    consumed_sources: vec![newer.identity.clone()],
                    retired_sources: Vec::new(),
                    retired: None,
                    token_count: 0,
                    cleanup: Default::default(),
                    completed: true,
                },
            );
            drop(state);
            registry.complete_prepared_source(&completed);
            registry
        };
        let eager = make();
        super::super::refreshed_rejoin::infallible(
            eager
                .state
                .lock()
                .unwrap()
                .prune_completed_custody(&mut OracleWalk::after_effect()),
        );
        eager
            .bind_prepared_output_source(&current_commit, selected.clone())
            .unwrap();
        let deferred = make();
        assert!(
            deferred.state.lock().unwrap().source_custody[&completed_commit]
                .source
                .is_none()
        );
        assert_eq!(
            deferred.retained_source_custody_count(),
            1,
            "a completed cleanup note retains no performed source"
        );
        assert!(deferred.state.lock().unwrap().pending_custody_prune());
        deferred
            .bind_prepared_output_source(&current_commit, selected.clone())
            .unwrap();
        let a = eager.state.lock().unwrap();
        let b = deferred.state.lock().unwrap();
        assert!(a.source_custody[&current_commit]
            .source_denial(&selected.identity)
            .is_none());
        assert_eq!(
            b.source_custody[&current_commit].retired_sources,
            a.source_custody[&current_commit].retired_sources,
            "seed {seed}"
        );
        assert_eq!(b.source_custody.len(), a.source_custody.len());
        assert_eq!(
            (
                b.record_retained_bytes
                    .load(std::sync::atomic::Ordering::Acquire),
                b.required_reserved_bytes,
                b.obligation_reserved_bytes
            ),
            (
                a.record_retained_bytes
                    .load(std::sync::atomic::Ordering::Acquire),
                a.required_reserved_bytes,
                a.obligation_reserved_bytes
            )
        );
        assert!(b.records.is_empty());
    }
}
