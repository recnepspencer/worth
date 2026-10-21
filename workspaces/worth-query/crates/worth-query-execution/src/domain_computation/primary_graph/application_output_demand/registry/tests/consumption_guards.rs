//! Root consumption is returned only for an abandoned, undelivered answer.
//! Receipts and source payloads come from real commits; stages are assigned
//! at the registry seam that owns disposal, not inferred from a handler run.
use super::super::{
    BoundOutputSource, PreparedOutputRootKind, PreparedReadyBacking, SourceCustody,
    WorthQueryOutputCheckpoint, WorthQueryOutputProgress, WorthQueryPendingOutputDelivery,
    WorthQueryPerformedOutputDemandSource,
};
use super::*;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support;
use crate::equality_type_id::EqualityTypeId;

fn retained_source(identity: super::super::SourceEpoch) -> SourceCustody {
    let mut receipt = recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation()
        .unwrap();
    let source = WorthQueryPerformedOutputDemandSource {
        receipt,
        change,
        observation,
        output_source_identity: None,
    };
    let scope = source_custody::real_scope_and_occurrence().0;
    SourceCustody {
        occurrence: source.receipt.product_branch().occurrence(),
        root_kind: PreparedOutputRootKind::Discovered(EqualityTypeId::of::<()>()),
        source: Some(source),
        discovery: None,
        bound_sources: Some(vec![BoundOutputSource { scope, identity }]),
        consumed_sources: Vec::new(),
        retired_sources: Vec::new(),
        retired: None,
        token_count: 1,
        cleanup: Default::default(),
        completed: false,
    }
}

fn repeat_source(initial: &SourceCustody) -> SourceCustody {
    SourceCustody {
        occurrence: initial.occurrence,
        root_kind: initial.root_kind.clone(),
        source: initial.source.clone(),
        discovery: None,
        bound_sources: initial.bound_sources.clone(),
        consumed_sources: Vec::new(),
        retired_sources: Vec::new(),
        retired: None,
        token_count: 1,
        cleanup: Default::default(),
        completed: false,
    }
}

#[test]
fn completed_and_retired_custody_return_no_consumption() {
    let identity = key("guarded-root", 5, 1).source;
    let initial = retained_source(identity.clone());
    let scope = initial.bound_sources.as_ref().unwrap()[0].scope;
    for completed in [false, true] {
        let mut custody = repeat_source(&initial);
        custody.consumed_sources.push(identity.clone());
        if completed {
            custody.completed = true;
        } else {
            // Keep the payload so this row independently pins the retired guard.
            custody.retired = Some(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "retired root",
            ));
        }
        custody.return_undelivered_root(&identity, scope);
        assert_eq!(
            custody.consumed_sources.as_slice(),
            std::slice::from_ref(&identity)
        );
    }
}

#[test]
fn delivered_and_ready_roots_keep_their_consumption_on_removal() {
    let row_key = key("delivered-root", 5, 1);
    let initial = retained_source(row_key.source.clone());
    let source = initial.source.as_ref().unwrap();
    let commit = source.change.product_commit().clone();
    let scope = initial.bound_sources.as_ref().unwrap()[0].scope;
    for ready in [false, true] {
        let mut state = DemandRegistryState::default();
        let mut custody = repeat_source(&initial);
        custody.consumed_sources.push(row_key.source.clone());
        state.source_custody.insert(commit.clone(), custody);
        let stage = if ready {
            closed_retirement::ready()
        } else {
            let backing = PreparedReadyBacking::new();
            DemandState::Output(WorthQueryOutputProgress::new(
                WorthQueryOutputCheckpoint::Delivered {
                    receipt: source.receipt.clone(),
                    delivery: None,
                    ready_backing: backing,
                },
            ))
        };
        let mut row = record(initial.occurrence, stage, 0);
        row.source_scope = Some(scope);
        row.source_commits.push(commit.clone());
        state.records.insert(row_key.clone(), row);
        drop(state.remove_record(&row_key).unwrap());
        assert_eq!(
            state.source_custody[&commit].consumed_sources.as_slice(),
            std::slice::from_ref(&row_key.source),
            "ready={ready}: delivered work cannot be admitted again"
        );
    }
}

#[test]
fn an_abandoned_join_returns_the_requested_source_it_consumed() {
    let older = key_with_identity("joined-root", 5, 1, 90);
    let requested = key_with_identity("joined-root", 6, 1, 90);
    assert_ne!(older.source, requested.source);
    assert!(older.same_semantic_source(&requested));
    let initial = retained_source(requested.source.clone());
    let source = initial.source.as_ref().unwrap();
    let commit = source.change.product_commit().clone();
    let scope = initial.bound_sources.as_ref().unwrap()[0].scope;
    let registry = WorthQueryOutputDemandRegistry::default();
    {
        let mut state = registry.state.lock().unwrap();
        state
            .source_custody
            .insert(commit.clone(), repeat_source(&initial));
        let mut row = record(initial.occurrence, DemandState::Admitted, 0);
        row.source_scope = Some(scope);
        state.records.insert(older.clone(), row);
    }
    let (interest, _) = registry
        .admit(
            requested.clone(),
            Some(super::super::SelectedOutputAdmission::new(&commit, None)),
            scope,
            initial.occurrence,
            super::super::DemandAdmissionKind::Required,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        interest.key, older,
        "the admission actually joins the older key"
    );
    let mut state = registry.state.lock().unwrap();
    assert_eq!(
        state.source_custody[&commit].consumed_sources.as_slice(),
        std::slice::from_ref(&requested.source)
    );
    let backing = PreparedReadyBacking::new();
    let mut row = state.records.get_mut(&older).unwrap();
    row.state = DemandState::Output(WorthQueryOutputProgress::new(
        WorthQueryOutputCheckpoint::Published {
            receipt: source.receipt.clone(),
            delivery: WorthQueryPendingOutputDelivery::NoChange,
            ready_backing: backing,
        },
    ));
    row.performed_source = None; // The real publication finish releases this payload.
    row.interests = 0;
    row.finish();
    drop(state.remove_record(&older).unwrap());
    assert!(
        state.source_custody[&commit].consumed_sources.is_empty(),
        "the returned identity is the requested source, not the older row key"
    );
    drop(state);
    drop(interest);
}
