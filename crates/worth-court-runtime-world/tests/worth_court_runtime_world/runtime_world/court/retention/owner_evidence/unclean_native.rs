//! A kill between World open, Native effect and World occurrence reopens to
//! exactly the state its durable facts determine (C1.1 bounded property).
use super::super::artifact_source::Fixture;
use super::*;
use proptest::{prelude::*, test_runner::FileFailurePersistence};
use worth_relational::facade::{
    history::RelationalCommitReceipt, mvcc::RelationalTransactionIntent,
};
mod refusals;

#[derive(Clone, Copy, Debug)]
enum Step {
    /// Open, Native effect and World occurrence are all durable.
    Acknowledged,
    /// The process dies after the durable open, before any owner effect.
    OpenOnly,
    /// The process dies after the durable Native effect, before the occurrence.
    EffectOnly,
    Reopen,
    /// Continue each recovered effect; release each effect-free original.
    Resolve,
}

/// One unfinished original: attempt, reserved commit and custody ordinals,
/// and the Native receipt its owner made durable, if any.
#[derive(Clone, Debug, PartialEq)]
struct Original {
    identities: [u64; 3],
    receipt: Option<RelationalCommitReceipt>,
}

/// What the recorded durable facts alone determine.
#[derive(Debug, Default)]
struct Model {
    generation: u64,
    native_commits: usize,
    originals: Vec<Original>,
    /// Largest attempt, commit and custody ordinal issued so far.
    frontier: Option<[u64; 3]>,
    writes: u32,
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "proptest-regressions/unclean_native.txt",
        ))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn killed_world_publication_reopens_to_its_recorded_facts(
        steps in proptest::collection::vec(
            prop_oneof![
                Just(Step::Acknowledged),
                Just(Step::OpenOnly),
                Just(Step::EffectOnly),
                Just(Step::Reopen),
                Just(Step::Resolve),
            ],
            1..=6,
        )
    ) {
        run(&steps);
    }
}

fn run(steps: &[Step]) {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(_) => {}
        other => panic!("real bootstrap: {other:?}"),
    }
    let mut model = Model::default();
    for step in steps.iter().copied() {
        match step {
            Step::Reopen => fixture = model.reopen(fixture, &request),
            Step::Resolve => model.resolve(&fixture, &request),
            // Fresh work waits until every recovered original is decided.
            _ if !model.originals.is_empty() => model.resolve(&fixture, &request),
            kind => {
                if model.attempt(&fixture, &request, kind) {
                    // The crash ends the process: only a fresh one continues.
                    fixture = model.reopen(fixture, &request);
                }
            }
        }
    }
    model.resolve(&fixture, &request);
    // The next fresh identities lie beyond every issued one, unfinished opens included.
    assert!(!model.attempt(&fixture, &request, Step::Acknowledged));
    let fixture = model.reopen(fixture, &request);
    fixture.finish(&request);
}

impl Model {
    /// Returns whether the step ended in a crash.
    fn attempt(
        &mut self,
        fixture: &Fixture,
        request: &ExecutionRequest<'_, '_>,
        kind: Step,
    ) -> bool {
        self.writes += 1;
        let head = root(fixture);
        let cancellation = RuntimeWorldCancellationSource::new();
        let port = fixture.world.publication_port();
        let opened = match port.open_publication_without_signal(
            request,
            head.clone(),
            fixture.recorder.program_source(&head),
            RelationalTransactionIntent::ordinary(),
            &cancellation.token(),
            None,
        ) {
            WorldOpenAttemptOutcome::Opened(value) => value,
            other => panic!("durable open: {other:?}"),
        };
        let mut transaction =
            worth_testkit_relational::aspect_storage::publications::begin_main_transaction(
                &fixture.native,
            );
        transaction
            .push_batch(
                worth_testkit_relational::durable_publication::batch_create(&format!(
                    "model-{}",
                    self.writes
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let proposal = transaction.validate(request, &fixture.native).unwrap();
        let prepared = opened
            .prepare_native(&fixture.native, request, proposal)
            .unwrap();
        let identities = [
            prepared.attempt_identity().ordinal(),
            prepared.reserved_commit_identity().ordinal(),
            prepared.unpublished_recovery_handle().identity().ordinal(),
        ];
        if let Some(frontier) = self.frontier {
            for (issued, before) in identities.iter().zip(frontier) {
                assert!(
                    *issued > before,
                    "identity {issued} reissued at or below {before}"
                );
            }
        }
        self.frontier = Some(identities);
        if let Step::OpenOnly = kind {
            fixture.recorder.crash();
            drop(prepared);
            self.originals.push(Original {
                identities,
                receipt: None,
            });
            return true;
        }
        if let Step::EffectOnly = kind {
            fixture.recorder.crash();
        }
        let outcome =
            port.execute_durable_without_signal(*request, prepared, &cancellation.token());
        self.native_commits += 1;
        match (kind, outcome) {
            (Step::Acknowledged, WorldDurablePublicationOutcome::Performed(performed)) => {
                drop(performed.consume());
                self.generation += 1;
                false
            }
            (
                Step::EffectOnly,
                WorldDurablePublicationOutcome::RecoveryRequired { custody, .. },
            ) => {
                let receipt = custody
                    .component_results()
                    .relational_settlement()
                    .cloned()
                    .expect("the Native effect precedes the lost occurrence");
                self.originals.push(Original {
                    identities,
                    receipt: Some(receipt),
                });
                true
            }
            (kind, other) => panic!("{kind:?} reached {other:?}"),
        }
    }

    fn reopen(&self, fixture: Fixture, request: &ExecutionRequest<'_, '_>) -> Fixture {
        let fixture = fixture.reopen(request);
        self.compare(&fixture);
        fixture
    }

    /// The readmitted catalog, head and retry posture equal the model's.
    fn compare(&self, fixture: &Fixture) {
        let head = root(fixture);
        assert_eq!(head.reference_generation().get(), self.generation);
        let inspection = fixture.world.inspection_port();
        assert_eq!(
            inspection.history_snapshot().unwrap().installed_commits() as u64,
            self.generation + 1
        );
        assert_eq!(
            fixture.native.history().immutable_commit_count(),
            self.native_commits
        );
        let page = inspection.recovery_page(None, maximum()).unwrap();
        let recovery = fixture.world.recovery_port();
        let mut actual = Vec::new();
        for row in page.rows() {
            assert_eq!(row.state(), RuntimeWorldRecoveryRecordState::Recovered);
            let observe = || {
                let effects = recovery.inspect_recovered_effects(row.handle()).unwrap();
                assert_eq!(
                    effects.expected_head(),
                    &head,
                    "the live head stays at its predecessor"
                );
                Original {
                    identities: [
                        effects.attempt_identity().ordinal(),
                        effects.reserved_commit_identity().ordinal(),
                        effects.identity().ordinal(),
                    ],
                    receipt: effects.native_receipt(),
                }
            };
            let first = observe();
            assert_eq!(
                observe(),
                first,
                "a duplicate retry observes the same original"
            );
            actual.push(first);
        }
        let mut expected = self.originals.clone();
        actual.sort_by_key(|original| original.identities);
        expected.sort_by_key(|original| original.identities);
        assert_eq!(actual, expected);
    }

    fn resolve(&mut self, fixture: &Fixture, request: &ExecutionRequest<'_, '_>) {
        let page = fixture
            .world
            .inspection_port()
            .recovery_page(None, maximum())
            .unwrap();
        let recovery = fixture.world.recovery_port();
        for row in page.rows() {
            let effects = recovery.inspect_recovered_effects(row.handle()).unwrap();
            if effects.native_receipt().is_some() {
                let mut outcome = recovery.continue_recovered_effects(request, effects);
                for _ in 0..64 {
                    let WorldRecoveredContinuationOutcome::Pending(effects) = outcome else {
                        break;
                    };
                    outcome = recovery.continue_recovered_effects(request, effects);
                }
                assert!(
                    matches!(outcome, WorldRecoveredContinuationOutcome::Published { .. }),
                    "recovered Native effect continues once: {outcome:?}"
                );
                self.generation += 1;
            } else {
                let mut outcome = recovery.release_recovered_effects(request, effects);
                for _ in 0..64 {
                    let WorldRecoveredReleaseOutcome::Pending(effects) = outcome else {
                        break;
                    };
                    outcome = recovery.release_recovered_effects(request, effects);
                }
                assert!(
                    matches!(outcome, WorldRecoveredReleaseOutcome::Released(_)),
                    "an effect-free original releases: {outcome:?}"
                );
            }
        }
        self.originals.clear();
        drop(page);
        self.compare(fixture);
    }
}

fn root(fixture: &Fixture) -> ProductBranchObservation {
    fixture
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap()
}

fn maximum() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(16).unwrap()
}
