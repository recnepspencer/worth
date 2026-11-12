//! Registered terminal refusal is retried explicitly, without replacing the logical frame.
use super::*;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn registered_no_effect_initial_steps_retry_the_same_frames_without_repeating_initialization() {
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), None, Some(0)),
    );
    for refused_journal in [
        SignalDurableJournal::Definitions,
        SignalDurableJournal::Advances,
    ] {
        let directory = tempfile::tempdir().unwrap();
        let foreign_directory = tempfile::tempdir().unwrap();
        let (serving, medium) = media::open(directory.path(), true, &execution);
        let (foreign_serving, foreign_medium) =
            media::open(foreign_directory.path(), true, &execution);
        let observed =
            paused_medium::PausedMedium::with_foreign(medium.clone(), foreign_medium.clone());
        let mut pending = prepare(program::program().builder, observed.clone(), &execution);
        let definition = pending.definition().clone();
        let advance = pending.advance_artifact().clone();
        if refused_journal == SignalDurableJournal::Advances {
            let deadline = Instant::now() + Duration::from_secs(120);
            while !observed.branch_observed.load(Ordering::Relaxed) {
                assert_eq!(
                    pending.advance(&execution).unwrap(),
                    SignalInitialSourceProgress::Pending
                );
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
        }
        observed.pause_branch.store(false, Ordering::Relaxed);
        let limited = worth_execution::SerialRequest::from_policy(
            &policy,
            worth_execution::CancellationSource::new().token(),
            None,
        );
        let refusal = pending
            .advance(&ExecutionRequest::serial(&limited))
            .unwrap_err();
        assert_eq!(refusal.journal, refused_journal);
        assert!(matches!(
            refusal.fate,
            DurablePublicationFate::ProvenNoEffect {
                detail: SignalDurableAdvanceDenial::Medium(SignalDurableMediumDenial::Request(_))
            }
        ));
        // A healthy ordinary observation still returns the registered terminal.
        // Pre-registration rejection cannot produce this cached Unavailable.
        let cached = match refused_journal {
            SignalDurableJournal::Definitions => medium.advance_definition(&definition, &execution),
            SignalDurableJournal::Advances => medium.advance_branch(&advance, &execution),
        };
        assert!(
            matches!(cached, SignalPublicationObservation::Finished(done) if matches!(done.fate(),
            DurablePublicationFate::ProvenNoEffect { detail: SignalDurableMediumDenial::Unavailable }))
        );
        assert_eq!(pending.definition().as_bytes(), definition.as_bytes());
        assert_eq!(pending.advance_artifact().as_bytes(), advance.as_bytes());
        finish(&mut pending, &execution);
        assert!(observed.retry_checked.load(Ordering::Relaxed));
        let seed = published(pending);
        assert_eq!(seed.source_point().reference(), advance.reference());
        let source = medium.capture_recovery_source(&execution).unwrap();
        assert_eq!(
            source
                .frontier(SignalDurableJournal::Definitions, &execution)
                .unwrap(),
            1
        );
        assert_eq!(
            source
                .frontier(SignalDurableJournal::Advances, &execution)
                .unwrap(),
            1
        );
        let foreign_source = foreign_medium.capture_recovery_source(&execution).unwrap();
        assert_eq!(
            foreign_source
                .frontier(SignalDurableJournal::Definitions, &execution)
                .unwrap(),
            0
        );
        assert_eq!(
            foreign_source
                .frontier(SignalDurableJournal::Advances, &execution)
                .unwrap(),
            0
        );
        drop((source, foreign_source, seed, observed));
        media::close(serving, medium, &execution);
        media::close(foreign_serving, foreign_medium, &execution);
    }
}
