//! Committed claims queue terminal members; the next request owns their walk.
use super::super::WorthQueryOutputCheckpoint;
use super::*;

#[test]
fn publication_queues_released_counts_then_drain_matches_eager_member_removal() {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    use crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner;
    let owner = SourceInvalidationOwner::new(
        WorthQueryInvalidationResources::install(
            WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
        )
        .unwrap(),
        1,
    );
    let (_lineage, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    for seed in 0..16 {
        let (queued, selected, held) = published(seed, &identity, &owner, None);
        let (eager, _, _) = published(seed, &identity, &owner, None);
        // At 337d6d4b88 this member removal happened synchronously after the
        // decrement, before the same queue push. Its result is independent
        // of pending_cleanup_queued, so reproduce that eager decision here.
        let mut eager_state = eager.state.lock().unwrap();
        super::super::refreshed_rejoin::infallible(
            eager_state
                .remove_required_member_if_released(&selected, &mut OracleWalk::after_effect()),
        );
        eager_state
            .drain_terminal_cleanup(&mut record_admission())
            .unwrap();
        let mut state = queued.state.lock().unwrap();
        assert_eq!(state.records[&selected].framework_required_count, 0);
        assert!(state.records[&selected].pending_cleanup_queued);
        assert!(state.required_keys.contains(&selected));
        state
            .drain_terminal_cleanup(&mut record_admission())
            .unwrap();
        assert_eq!(state.required_keys.contains(&selected), held);
        assert_eq!(
            state
                .records
                .iter()
                .map(|(key, row)| (
                    key.clone(),
                    row.framework_required_count,
                    row.interests,
                    row.prerequisites.len(),
                    row.pending_cleanup_queued
                ))
                .collect::<Vec<_>>(),
            eager_state
                .records
                .iter()
                .map(|(key, row)| (
                    key.clone(),
                    row.framework_required_count,
                    row.interests,
                    row.prerequisites.len(),
                    row.pending_cleanup_queued
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(state.required_keys, eager_state.required_keys);
        assert_eq!(
            state.required_reserved_bytes,
            eager_state.required_reserved_bytes
        );
        assert_eq!(
            state.obligation_reserved_bytes,
            eager_state.obligation_reserved_bytes
        );
        assert_eq!(
            state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
            eager_state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire)
        );
    }
}

fn published(
    seed: u64,
    identity: &Arc<
        crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    >,
    owner: &crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner,
    completion: Option<&super::super::ReadyCompletion>,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandKey,
    bool,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let upstream = key("published-upstream", seed + 1, 1);
    let downstream = key("published-downstream", seed + 1, 2);
    let held = seed & 4 != 0;
    let mut prior = record(
        occurrence(),
        DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Closed,
            "",
        )),
        usize::from(held),
    );
    if let Some(completion) = completion {
        prior.state = DemandState::Output(super::super::WorthQueryOutputProgress::new(
            WorthQueryOutputCheckpoint::Ready(completion.clone()),
        ));
    }
    prior.framework_required_count = 1;
    prior.prerequisites.reserve_exact(1);
    let mut row = record(occurrence(), DemandState::Running, 1);
    row.required_interests = 1;
    row.prerequisites.push(Arc::new(upstream.clone()));
    let wake = Arc::clone(&row.wake);
    {
        let mut state = registry.state.lock().unwrap();
        state.required_reserved_bytes = (prior.prerequisites.capacity()
            + row.prerequisites.capacity())
            * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
        state.records.insert(upstream.clone(), prior);
        state.records.insert(downstream.clone(), row);
        for offset in 0..seed % 4 {
            state.records.insert(
                key("published-upstream", seed + 2 + offset, 1),
                record(occurrence(), DemandState::Admitted, 0),
            );
        }
        let member = state.prepare_required_member(&upstream).unwrap();
        state.install_required_member(member);
    }
    let interest = required_interest(&registry, downstream, wake);
    let mut admission = record_admission();
    let context = interest.required_context(owner, &mut admission).unwrap();
    let mut claims = context
        .prepare_prerequisites(std::iter::empty(), owner, &mut admission)
        .unwrap();
    claims.reserve_identity(identity, &mut admission).unwrap();
    let _superseded = claims.publish(Arc::clone(identity));
    // The terminal upstream queue is preserved by the interest's Drop. The
    // downstream is still live until this function returns its registry.
    drop(interest);
    (registry, upstream, held)
}

#[test]
fn registry_ready_upstream_declines_queue_and_published_member_walk_completes() {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    let owner =
        crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner::new(
            WorthQueryInvalidationResources::install(
                WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
            )
            .unwrap(),
            1,
        );
    let (_lineage, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let DemandState::Output(output) = super::closed_retirement::ready() else {
        panic!("Ready fixture")
    };
    let Some(WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else {
        panic!("Ready fixture")
    };
    let (registry, selected, held) = published(0, &identity, &owner, Some(&completion));
    let mut state = registry.state.lock().unwrap();
    assert!(!held);
    assert!(state.records[&selected].has_cached_ready());
    assert_eq!(state.records[&selected].framework_required_count, 0);
    assert!(!state.records[&selected].pending_cleanup_queued);
    assert!(!state.defer_terminal_cleanup(&Arc::new(selected.clone()), 0));
    assert!(!state.required_keys.contains(&selected));
}
