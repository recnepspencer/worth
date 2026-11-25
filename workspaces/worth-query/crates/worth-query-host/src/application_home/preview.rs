//! Expected-source preview forks ("what-if" branches) with atomic adoption and discard.
use std::fmt;
use std::time::{Duration, Instant};

use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranchAdmissionDenial,
    },
    product::{
        WorthQueryExpectedProductBranch, WorthQueryExpectedProductBranchAdoptionDenial,
        WorthQueryExpectedProductBranchCreationContinuation, WorthQueryProductBranch,
        WorthQueryProductBranchCloseReceipt, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchCreationRecovery, WorthQueryProductBranchReadIdentity,
        WorthQueryProductBranchRecoveryDenial,
    },
};

use super::WorthQueryApplicationHome;

const FORK_CONTINUATION_TIMEOUT: Duration = Duration::from_secs(15);
const FORK_CONTINUATION_RETRY_DELAY: Duration = Duration::from_millis(10);

/// A live preview branch forked from the Home's exact active head.
///
/// Mutate and query it via `home.session(me).on_branch(preview.branch())`, then
/// either promote it with [`WorthQueryApplicationHome::adopt_preview`] or throw
/// it away cleanly with [`Self::discard`].
#[must_use = "adopt or discard the preview fork before closing the Home"]
pub struct WorthQueryPreviewFork<'home, Schema: ApplicationSchema> {
    label: String,
    expected: WorthQueryExpectedProductBranch<'home, Schema>,
}

impl<'home, Schema: ApplicationSchema> WorthQueryPreviewFork<'home, Schema> {
    /// Human-readable label attached to this preview fork.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The product branch selector to pass into `session.on_branch(preview.branch())`.
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.expected.branch()
    }

    /// Descriptive identity of the exact source head this preview was forked from.
    pub fn expected_source(&self) -> WorthQueryProductBranchReadIdentity {
        self.expected.expected_source()
    }

    /// Unwraps the underlying expected-source branch owner for advanced control.
    pub fn into_expected(self) -> WorthQueryExpectedProductBranch<'home, Schema> {
        self.expected
    }

    /// Discards this preview fork cleanly without modifying the Home's active head.
    pub fn discard<Program>(
        self,
        home: &'home WorthQueryApplicationHome<Schema, Program>,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryPreviewDiscardFailure<'home, Schema>>
    {
        let Self { label, expected } = self;
        let mut slot = Some(expected);
        let scope = fresh_preview_scope();
        let result = home.primary_graph().with_funded_application_completion(
            &scope,
            ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                let candidate = slot.take().expect("advancement runs once");
                candidate.close(&phase)
            },
        );
        match result {
            Ok(Ok(receipt)) => Ok(receipt),
            Ok(Err(failure)) => {
                let (candidate, denial) = failure.into_parts();
                Err(WorthQueryPreviewDiscardFailure {
                    preview: Self {
                        label,
                        expected: candidate,
                    },
                    denial: WorthQueryPreviewDiscardDenial::Close(denial),
                })
            }
            Err(denial) => Err(WorthQueryPreviewDiscardFailure {
                preview: Self {
                    label,
                    expected: slot.take().expect("failed advancement preserved preview"),
                },
                denial: WorthQueryPreviewDiscardDenial::Advancement(denial),
            }),
        }
    }
}

/// Why creating a preview fork was refused.
#[derive(Debug)]
pub enum WorthQueryPreviewForkDenial {
    Advancement(WorthQueryAdvancementDenial),
    Selection(WorthQueryProductBranchAdmissionDenial),
    Create(WorthQueryProductBranchCreateError),
    Recovery {
        denial: WorthQueryProductBranchRecoveryDenial,
        recovery: WorthQueryProductBranchCreationRecovery,
    },
    ContinuationNotPerformed,
}

/// Why adopting a preview fork as the Home's active head was refused.
#[derive(Debug)]
pub enum WorthQueryPreviewAdoptDenial {
    Advancement(WorthQueryAdvancementDenial),
    Adoption(WorthQueryExpectedProductBranchAdoptionDenial),
}

impl WorthQueryPreviewAdoptDenial {
    /// Returns `true` when adoption was refused because the Home's active head
    /// advanced or its committed source basis changed since this preview forked.
    pub fn is_stale_head(&self) -> bool {
        use worth_relational::facade::branch::RelationalBranchBasisDenial as Basis;
        matches!(
            self,
            Self::Adoption(
                WorthQueryExpectedProductBranchAdoptionDenial::HeadMoved
                    | WorthQueryExpectedProductBranchAdoptionDenial::HeadReplaced
                    | WorthQueryExpectedProductBranchAdoptionDenial::SourceBasis(
                        Basis::StaleReferenceGeneration
                            | Basis::WrongBranchLocalTruthVersion
                            | Basis::EmptyCommittedTargetMismatch
                            | Basis::UnavailableRetainedTarget
                    )
            )
        )
    }
}

/// Failed adoption retains the preview fork intact so the caller can inspect
/// the refusal and either retry or discard it.
#[must_use = "retain or discard the preview fork returned by failed adoption"]
pub struct WorthQueryPreviewAdoptFailure<'home, Schema: ApplicationSchema> {
    preview: WorthQueryPreviewFork<'home, Schema>,
    denial: WorthQueryPreviewAdoptDenial,
}

impl<'home, Schema: ApplicationSchema> WorthQueryPreviewAdoptFailure<'home, Schema> {
    pub fn preview(&self) -> &WorthQueryPreviewFork<'home, Schema> {
        &self.preview
    }

    pub fn denial(&self) -> &WorthQueryPreviewAdoptDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPreviewFork<'home, Schema>,
        WorthQueryPreviewAdoptDenial,
    ) {
        (self.preview, self.denial)
    }
}

impl<Schema: ApplicationSchema> fmt::Debug for WorthQueryPreviewAdoptFailure<'_, Schema> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthQueryPreviewAdoptFailure")
            .field("label", &self.preview.label)
            .field("branch", &self.preview.branch())
            .field("denial", &self.denial)
            .finish()
    }
}

/// Why discarding a preview fork was refused.
#[derive(Debug)]
pub enum WorthQueryPreviewDiscardDenial {
    Advancement(WorthQueryAdvancementDenial),
    Close(WorthQueryApplicationProductBranchCloseDenial),
}

/// Failed discard retains the preview fork intact for retry or inspection.
#[must_use = "retain or retry discarding the preview fork"]
pub struct WorthQueryPreviewDiscardFailure<'home, Schema: ApplicationSchema> {
    preview: WorthQueryPreviewFork<'home, Schema>,
    denial: WorthQueryPreviewDiscardDenial,
}

impl<'home, Schema: ApplicationSchema> WorthQueryPreviewDiscardFailure<'home, Schema> {
    pub fn preview(&self) -> &WorthQueryPreviewFork<'home, Schema> {
        &self.preview
    }

    pub fn denial(&self) -> &WorthQueryPreviewDiscardDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPreviewFork<'home, Schema>,
        WorthQueryPreviewDiscardDenial,
    ) {
        (self.preview, self.denial)
    }
}

impl<Schema: ApplicationSchema> fmt::Debug for WorthQueryPreviewDiscardFailure<'_, Schema> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthQueryPreviewDiscardFailure")
            .field("label", &self.preview.label)
            .field("branch", &self.preview.branch())
            .field("denial", &self.denial)
            .finish()
    }
}

fn fresh_preview_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    )
}

pub(super) fn fork_expected_from_active<'home, Schema: ApplicationSchema, Program>(
    home: &'home WorthQueryApplicationHome<Schema, Program>,
    clock: &WorthQueryAdmissionClock,
) -> Result<WorthQueryExpectedProductBranch<'home, Schema>, WorthQueryPreviewForkDenial> {
    use WorthQueryExpectedProductBranchCreationContinuation as Continued;
    let active = home.active_branch();
    let graph = home.primary_graph();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    let initial = graph
        .with_funded_application_completion(
            &scope,
            ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                let selected = graph
                    .on_branch(active)
                    .select(&phase)
                    .map_err(WorthQueryPreviewForkDenial::Selection)?;
                let fork = graph
                    .branches()
                    .fork_expected(selected)
                    .map_err(WorthQueryPreviewForkDenial::Selection)?;
                fork.components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase)
                    .map_err(WorthQueryPreviewForkDenial::Create)
            },
        )
        .map_err(WorthQueryPreviewForkDenial::Advancement)?;
    let mut recovery = match initial {
        Ok(candidate) => return Ok(candidate),
        Err(WorthQueryPreviewForkDenial::Create(
            WorthQueryProductBranchCreateError::ProductUnpublished(original),
        )) => original,
        Err(other) => return Err(other),
    };
    let deadline = Instant::now() + FORK_CONTINUATION_TIMEOUT;
    while Instant::now() < deadline {
        let retry_scope = WorthQueryRequestScope::new(
            None,
            WorthQueryCancellationSource::new().token(),
            clock.clone(),
        );
        let step = graph
            .with_funded_application_completion(
                &retry_scope,
                ExecutionAllocationPolicy::SystemAllocation,
                |phase| graph.branches().continue_expected(recovery, &phase),
            )
            .map_err(WorthQueryPreviewForkDenial::Advancement)?;
        match step {
            Ok(Continued::Performed(candidate)) => return Ok(candidate),
            Ok(Continued::NoEffect(_) | Continued::ReleasedOriginal(_)) => {
                return Err(WorthQueryPreviewForkDenial::ContinuationNotPerformed);
            }
            Err(failure) if failure.denial() == WorthQueryProductBranchRecoveryDenial::Busy => {
                recovery = failure.into_recovery();
                std::thread::sleep(FORK_CONTINUATION_RETRY_DELAY);
            }
            Err(failure) => {
                return Err(WorthQueryPreviewForkDenial::Recovery {
                    denial: failure.denial(),
                    recovery: failure.into_recovery(),
                });
            }
        }
    }
    Err(WorthQueryPreviewForkDenial::Recovery {
        denial: WorthQueryProductBranchRecoveryDenial::Busy,
        recovery,
    })
}

impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHome<Schema, Program> {
    /// Forks a new preview branch from the Home's exact current active head,
    /// completing Directory Home product publication when needed.
    pub fn fork_preview(
        &self,
        label: impl Into<String>,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        let label = label.into();
        let clock = WorthQueryAdmissionClock::system();
        let expected = fork_expected_from_active(self, &clock)?;
        Ok(WorthQueryPreviewFork { label, expected })
    }

    /// Atomically adopts a preview fork as this Home instance's active head if
    /// and only if the source head has not moved since the preview was forked.
    ///
    /// On success returns the previous active branch; subsequent
    /// [`Self::session`] calls target the adopted branch. Reopening a Directory
    /// Home begins on the durable root product branch. On refusal returns the
    /// preview fork intact alongside the exact adoption denial (check
    /// [`WorthQueryPreviewAdoptDenial::is_stale_head`]).
    pub fn adopt_preview<'home>(
        &'home self,
        preview: WorthQueryPreviewFork<'home, Schema>,
    ) -> Result<WorthQueryProductBranch, WorthQueryPreviewAdoptFailure<'home, Schema>> {
        let WorthQueryPreviewFork { label, expected } = preview;
        let mut slot = Some(expected);
        let scope = fresh_preview_scope();
        let result = self.primary_graph().with_funded_application_completion(
            &scope,
            ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                let candidate = slot.take().expect("advancement runs once");
                let mut head = self
                    .active_head
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                candidate.adopt(&mut head, &phase)
            },
        );
        match result {
            Ok(Ok(previous)) => Ok(previous),
            Ok(Err(failure)) => {
                let (candidate, denial) = failure.into_parts();
                Err(WorthQueryPreviewAdoptFailure {
                    preview: WorthQueryPreviewFork {
                        label,
                        expected: candidate,
                    },
                    denial: WorthQueryPreviewAdoptDenial::Adoption(denial),
                })
            }
            Err(denial) => Err(WorthQueryPreviewAdoptFailure {
                preview: WorthQueryPreviewFork {
                    label,
                    expected: slot.take().expect("failed advancement preserved preview"),
                },
                denial: WorthQueryPreviewAdoptDenial::Advancement(denial),
            }),
        }
    }

    /// Closes a previously replaced branch (such as the branch returned by
    /// [`Self::adopt_preview`]) cleanly under application completion.
    pub fn close_branch(
        &self,
        branch: WorthQueryProductBranch,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryPreviewDiscardDenial> {
        let scope = fresh_preview_scope();
        self.primary_graph()
            .with_funded_application_completion(
                &scope,
                ExecutionAllocationPolicy::SystemAllocation,
                |phase| {
                    self.primary_graph()
                        .on_branch(branch)
                        .close(&phase)
                        .map_err(WorthQueryPreviewDiscardDenial::Close)
                },
            )
            .map_err(WorthQueryPreviewDiscardDenial::Advancement)?
    }
}
