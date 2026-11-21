//! What the recorded durable facts alone determine, and the real reopen
//! observations compared against it.
use super::*;
use worth_relational::facade::{
    history::RelationalCommitReceipt, mvcc::RelationalTransactionIntent,
};

/// One original: attempt, reserved commit and custody ordinals, the Native
/// receipt its owner made durable, if any, and whether Signal's effect is
/// durable (only a mixed original has one).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Original {
    pub identities: [u64; 3],
    pub receipt: Option<RelationalCommitReceipt>,
    pub signal: bool,
}

#[derive(Debug, Default)]
pub(super) struct Model {
    pub generation: u64,
    pub native_commits: usize,
    /// Durable Signal advances beyond the bootstrap's.
    pub signal_commits: usize,
    /// Signal's durable advance count once the Home is bootstrapped.
    pub signal_base: usize,
    /// Identities of mixed conditional-definition originals.
    pub mixed: Vec<[u64; 3]>,
    /// Recovered originals without a durable decision.
    pub originals: Vec<Original>,
    /// Originals with a durable release, effect-bearing or not.
    pub released: Vec<Original>,
    /// Recovered originals whose `Continued` disposition is durable but whose
    /// occurrence is not: they may only continue.
    pub continued: Vec<[u64; 3]>,
    /// A Native effect was released. Native's own branch still carries it;
    /// only the Store Home restores the World-selected Native head (see
    /// world_durable_release::graceful_source), so here fresh work refuses.
    pub stale: bool,
    /// Largest attempt, commit and custody ordinal issued so far.
    frontier: Option<[u64; 3]>,
    pub(super) writes: u32,
}

impl Model {
    /// The facts of a freshly bootstrapped Home.
    pub fn bootstrapped(fixture: &Fixture) -> Self {
        Self {
            signal_base: fixture.signal().published_advance_count().unwrap(),
            ..Self::default()
        }
    }
    /// An undecided performed Native effect fences fresh work on its branch:
    /// Native refuses the World head's basis before any new effect. (Keys are
    /// fenced by Query, not World.)
    pub fn assert_fenced(&self, fixture: &Fixture) {
        if self
            .originals
            .iter()
            .all(|original| original.receipt.is_none())
        {
            return;
        }
        let head = root(fixture);
        assert!(matches!(
            fixture.native.begin_branch_transaction(
                head.basis().relational_basis(),
                RelationalTransactionIntent::ordinary(),
            ),
            Err(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::StaleBasis)
        ));
    }

    /// Fresh work after a released Native effect refuses before any effect.
    pub fn fresh(
        &mut self,
        fixture: Fixture,
        request: &ExecutionRequest<'_, '_>,
        kind: Step,
    ) -> Fixture {
        let mut fixture = fixture;
        if self.stale {
            let head = root(&fixture);
            assert!(matches!(
                fixture.native.begin_branch_transaction(
                    head.basis().relational_basis(),
                    RelationalTransactionIntent::ordinary(),
                ),
                Err(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::StaleBasis)
            ));
            drop(head);
            return fixture;
        }
        if self.attempt(&fixture, request, kind) {
            // The crash ends the process: only a fresh one continues.
            fixture = self.reopen(fixture, request);
        }
        fixture
    }

    /// Returns whether the step ended in a crash.
    pub(super) fn attempt(
        &mut self,
        fixture: &Fixture,
        request: &ExecutionRequest<'_, '_>,
        kind: Step,
    ) -> bool {
        self.writes += 1;
        if let Step::Mixed(cut) = kind {
            return self.attempt_mixed(fixture, request, cut);
        }
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
        let mut transaction = fixture
            .native
            .begin_branch_transaction(
                head.basis().relational_basis(),
                RelationalTransactionIntent::ordinary(),
            )
            .expect("the World head's own Native basis admits the transaction");
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
        self.admit_identities(identities);
        if let Step::OpenOnly = kind {
            fixture.recorder.crash();
            drop(prepared);
            self.originals.push(Original {
                identities,
                receipt: None,
                signal: false,
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
                    signal: false,
                });
                true
            }
            (kind, other) => panic!("{kind:?} reached {other:?}"),
        }
    }

    /// Each fresh original's identities lie beyond every issued one.
    pub(super) fn admit_identities(&mut self, identities: [u64; 3]) {
        if let Some(frontier) = self.frontier {
            for (issued, before) in identities.iter().zip(frontier) {
                assert!(
                    *issued > before,
                    "identity {issued} reissued at or below {before}"
                );
            }
        }
        self.frontier = Some(identities);
    }

    pub fn reopen(&mut self, fixture: Fixture, request: &ExecutionRequest<'_, '_>) -> Fixture {
        let fixture = fixture.reopen(request);
        self.compare(&fixture, request);
        fixture
    }

    /// The readmitted catalogs, head and retry posture equal the model's.
    pub fn compare(&self, fixture: &Fixture, request: &ExecutionRequest<'_, '_>) {
        let head = root(fixture);
        assert_eq!(head.reference_generation().get(), self.generation);
        let inspection = fixture.world.inspection_port();
        assert_eq!(
            inspection.history_snapshot().unwrap().installed_commits() as u64,
            self.generation + 1
        );
        assert_eq!(
            fixture.native.history().immutable_commit_count(),
            self.native_commits,
            "no decision repeats or adds an owner effect"
        );
        assert_eq!(
            fixture.signal().published_advance_count().unwrap(),
            self.signal_base + self.signal_commits,
            "each Signal leg is performed exactly once"
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
                let receipt = effects.native_receipt();
                let original = Original {
                    identities: [
                        effects.attempt_identity().ordinal(),
                        effects.reserved_commit_identity().ordinal(),
                        effects.identity().ordinal(),
                    ],
                    signal: effects.owner_effect_count() > usize::from(receipt.is_some()),
                    receipt,
                };
                assert_eq!(
                    effects.next_actions(),
                    self.next_actions(&original),
                    "the recovered original offers exactly its recorded facts' actions"
                );
                original
            };
            let first = observe();
            assert_eq!(
                observe(),
                first,
                "a duplicate retry observes the same original"
            );
            actual.push(first);
        }
        assert_eq!(sorted(actual), sorted(self.originals.clone()));
        let released = recovery
            .released_original_page(request, None, maximum())
            .unwrap();
        let actual = released
            .rows()
            .iter()
            .map(|row| {
                (
                    row.attempt_identity().ordinal(),
                    row.identity().ordinal(),
                    row.native_receipt().cloned(),
                )
            })
            .collect::<Vec<_>>();
        let expected = self
            .released
            .iter()
            .map(|original| {
                (
                    original.identities[0],
                    original.identities[2],
                    original.receipt.clone(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sorted_by_custody(actual),
            sorted_by_custody(expected),
            "a durable release keeps its original and any Native receipt"
        );
    }

    /// What a reopened original may do, from its recorded facts alone: an
    /// effect-free original only releases; a durable `Continued` only
    /// continues; any other owner effect (Native only, with or without its
    /// planned Signal leg pending, or both) continues or releases.
    fn next_actions(&self, original: &Original) -> &'static [ProductUnpublishedNextAction] {
        use ProductUnpublishedNextAction as Action;
        if self.continued.contains(&original.identities) {
            &[Action::Inspect, Action::ContinueOriginalPublication]
        } else if original.receipt.is_none() && !original.signal {
            &[
                Action::Inspect,
                Action::ReleaseObligations,
                Action::CloseOwner,
            ]
        } else {
            &[
                Action::Inspect,
                Action::ContinueOriginalPublication,
                Action::ReleaseObligations,
                Action::CloseOwner,
            ]
        }
    }

    /// Remove and return the model's original with these recovered identities.
    pub fn take(&mut self, identities: [u64; 3]) -> Original {
        let index = self
            .originals
            .iter()
            .position(|original| original.identities == identities)
            .expect("a recovered original the model recorded");
        self.originals.remove(index)
    }
}

fn sorted(mut originals: Vec<Original>) -> Vec<Original> {
    originals.sort_by_key(|original| original.identities);
    originals
}

type Released = (u64, u64, Option<RelationalCommitReceipt>);
fn sorted_by_custody(mut released: Vec<Released>) -> Vec<Released> {
    released.sort_by_key(|(attempt, custody, _)| (*attempt, *custody));
    released
}
