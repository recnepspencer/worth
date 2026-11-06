//! Acknowledged World publication retains actual custody through a real allocation refusal.
use super::progression::CurrentOriginal;
use super::*;
use worth_execution::{
    CancellationSource, LeaseDenial, MemoryLimitDenial, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
use worth_runtime_world::facade::{
    WorldDurableMediumDenial, WorldHomeTransitionEffectDenial as Denial,
    WorldHomeTransitionEffectProgress as Progress,
};

#[test]
fn an_acknowledged_original_retains_its_native_custody_through_allocation_refusal_and_retry() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (association, identities, receipt, _earlier) =
        native_original::publish(directory.path(), &execution);
    {
        let mut current = CurrentOriginal::open(directory.path(), &execution);
        current.until_acknowledged(&execution);
        assert_eq!(current.executions.load(Ordering::SeqCst), 0);
        let zero = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(0)),
            CancellationSource::new().token(),
            None,
        );
        current.claim = Some(match current.advance(&ExecutionRequest::serial(&zero)) {
            Outcome::Refused {
                cause:
                    Denial::Medium(WorldDurableMediumDenial::Request(WorkCeilingDenial::Admission(
                        LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
                            admitted: 0,
                            ..
                        }),
                    ))),
                custody,
            } => custody,
            other @ (Outcome::Continued(_)
            | Outcome::Pending(_)
            | Outcome::EvidenceRequired(_)
            | Outcome::Indeterminate(_)
            | Outcome::ReopenRequired(_)
            | Outcome::Refused { .. }) => {
                panic!("the real after-ACK allocation must refuse under zero memory: {other:?}")
            }
        });
        assert_eq!(
            current
                .claim
                .as_ref()
                .unwrap()
                .home_transition_effect_progress()
                .unwrap(),
            Some(Progress::Acknowledged)
        );
        assert_eq!(
            current.claim.as_ref().unwrap().native_receipt().as_ref(),
            Some(&receipt)
        );
        current.refuse_release_and_reclaim(&execution);
        assert_eq!(
            current
                .claim
                .as_ref()
                .unwrap()
                .home_transition_effect_progress()
                .unwrap(),
            Some(Progress::Acknowledged)
        );
        let publication = current.finish(&execution);
        assert_eq!(publication.attempt_identity().ordinal(), identities.0);
        assert_eq!(
            publication.after().selected_commit().ordinal(),
            identities.1
        );
        assert_eq!(publication.native_receipt(), &receipt);
        progression::assert_live_head(&current.fixture, &publication);
        assert_eq!(current.executions.load(Ordering::SeqCst), 0);
        let subjects = current
            .fixture
            .home
            .relational()
            .runtime()
            .durability_authority()
            .discover_associated_effects(&execution, &association)
            .unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].receipt(), &receipt);
        drop((
            subjects,
            publication,
            current.claim,
            current.contracts,
            current.grant,
        ));
        close(current.fixture.home, &execution);
    }
    let fixture = original::reopen(directory.path(), &execution);
    let selected = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(selected.reference_generation().get(), 1);
    assert_eq!(selected.selected_commit().ordinal(), identities.1);
    assert_eq!(
        selected.publication_attempt().unwrap().ordinal(),
        identities.0
    );
    assert_eq!(
        fixture
            .home
            .world()
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    drop(selected);
    close(fixture.home, &execution);
}
