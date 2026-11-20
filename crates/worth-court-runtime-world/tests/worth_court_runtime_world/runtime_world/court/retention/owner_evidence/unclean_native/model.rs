//! What the recorded durable facts alone determine, and the real reopen
//! observations compared against it.
use super::*;
use worth_relational::facade::{
    history::RelationalCommitReceipt, mvcc::RelationalTransactionIntent,
};

/// One original: attempt, reserved commit and custody ordinals, and the
/// Native receipt its owner made durable, if any.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Original {
    pub identities: [u64; 3],
    pub receipt: Option<RelationalCommitReceipt>,
}

#[derive(Debug, Default)]
pub(super) struct Model {
    pub generation: u64,
    pub native_commits: usize,
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
    writes: u32,
}

impl Model {
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
