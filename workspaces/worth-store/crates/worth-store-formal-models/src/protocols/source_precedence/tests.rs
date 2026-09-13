use super::{
    require_selectable_source, ModeledSourceCandidateRole, SourceAuthorityPosture,
    SourcePrecedenceAction, SourcePrecedenceActionKind, SourcePrecedenceDenial,
};
use worth_store_test_support::harness::recovery::deterministic_checkpoint_plus_tail_source;

#[test]
fn only_admitted_authority_is_selectable() {
    assert_eq!(
        require_selectable_source(SourceAuthorityPosture::AdmittedAuthority),
        Ok(())
    );
    assert_eq!(
        require_selectable_source(SourceAuthorityPosture::AdvisoryOnly),
        Err(SourcePrecedenceDenial::CandidateNotAdmitted)
    );
    for posture in [
        SourceAuthorityPosture::DerivedLocator,
        SourceAuthorityPosture::ReplayHelper,
    ] {
        assert_eq!(
            require_selectable_source(posture),
            Err(SourcePrecedenceDenial::DerivedSourceCannotBeAuthority)
        );
    }
    assert_eq!(
        require_selectable_source(SourceAuthorityPosture::Quarantined),
        Err(SourcePrecedenceDenial::QuarantinedSourceCannotBeSelected)
    );
}

#[test]
fn every_action_reports_its_exact_kind() {
    let actions = [
        SourcePrecedenceAction::CandidateDiscovered {
            discovery_order: 1,
            role: ModeledSourceCandidateRole::CheckpointBase,
        },
        SourcePrecedenceAction::CandidateAdmitted { discovery_order: 1 },
        SourcePrecedenceAction::CandidateAdvisoryOnly { discovery_order: 2 },
        SourcePrecedenceAction::CandidateRejected { discovery_order: 3 },
        SourcePrecedenceAction::ContradictionPreserved,
        SourcePrecedenceAction::SourceSelected,
        SourcePrecedenceAction::SourceQuarantined,
        SourcePrecedenceAction::SourceDenied,
    ];

    assert_eq!(
        actions.map(SourcePrecedenceAction::kind),
        SourcePrecedenceActionKind::all()
    );
}

#[test]
fn real_recovery_source_selection_preserves_checkpoint_and_wal_truth() {
    let selected = deterministic_checkpoint_plus_tail_source();
    let trace = selected.trace();

    assert!(trace.checkpoint_selected());
    assert_eq!(trace.wal_segments(), 1);
    assert!(!trace.interrupted_wal_tail());
    assert!(!trace.compaction_selected());
    assert_eq!(trace.residue_count(), 0);
    assert_eq!(
        require_selectable_source(SourceAuthorityPosture::AdmittedAuthority),
        Ok(())
    );
}
