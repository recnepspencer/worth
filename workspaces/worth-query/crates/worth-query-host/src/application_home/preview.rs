//! Expected-source preview forks ("what-if" branches) with atomic adoption,
//! custody-preserving continuation recovery, and active-branch close protection.
use std::time::Duration;

use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::product::{
    WorthQueryExpectedProductBranch, WorthQueryExpectedProductBranchCreationContinuation,
    WorthQueryProductBranch, WorthQueryProductBranchCloseReceipt,
    WorthQueryProductBranchCreateError, WorthQueryProductBranchCreationRecovery,
    WorthQueryProductBranchReadIdentity, WorthQueryProductBranchRecoveryDenial,
};

use super::WorthQueryApplicationHome;

mod denials;
pub use denials::{
    WorthQueryPreviewAdoptDenial, WorthQueryPreviewAdoptFailure, WorthQueryPreviewDiscardDenial,
    WorthQueryPreviewDiscardFailure, WorthQueryPreviewForkDenial,
};

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

fn fresh_preview_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    )
}

pub(super) fn continue_expected_recovery<'home, Schema: ApplicationSchema, Program>(
    home: &'home WorthQueryApplicationHome<Schema, Program>,
    mut recovery: WorthQueryProductBranchCreationRecovery,
    scope: &WorthQueryRequestScope,
) -> Result<WorthQueryExpectedProductBranch<'home, Schema>, WorthQueryPreviewForkDenial> {
    use WorthQueryExpectedProductBranchCreationContinuation as Continued;
    let graph = home.primary_graph();
    let deadline = worth_execution::MonotonicClock::system().now() + FORK_CONTINUATION_TIMEOUT;
    while worth_execution::MonotonicClock::system().now() < deadline {
        let mut slot = Some(recovery);
        let step = match graph.with_funded_application_completion(
            scope,
            ExecutionAllocationPolicy::SystemAllocation,
            |phase| {
                let current = slot.take().expect("advancement runs once");
                graph.branches().continue_expected(current, &phase)
            },
        ) {
            Ok(step) => step,
            Err(denial) => match slot.take() {
                Some(recovery) => {
                    return Err(WorthQueryPreviewForkDenial::ContinuationAdvancement {
                        denial,
                        recovery,
                    });
                }
                None => return Err(WorthQueryPreviewForkDenial::Advancement(denial)),
            },
        };
        match step {
            Ok(Continued::Performed(candidate)) => return Ok(candidate),
            Ok(Continued::NoEffect(receipt)) => {
                return Err(WorthQueryPreviewForkDenial::NoEffect(receipt));
            }
            Ok(Continued::ReleasedOriginal(receipt)) => {
                return Err(WorthQueryPreviewForkDenial::ReleasedOriginal(receipt));
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

pub(super) fn fork_expected_from_active_with_scope<'home, Schema: ApplicationSchema, Program>(
    home: &'home WorthQueryApplicationHome<Schema, Program>,
    scope: &WorthQueryRequestScope,
) -> Result<WorthQueryExpectedProductBranch<'home, Schema>, WorthQueryPreviewForkDenial> {
    let active = home.active_branch();
    let graph = home.primary_graph();
    let initial = graph
        .with_funded_application_completion(
            scope,
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
    let recovery = match initial {
        Ok(candidate) => return Ok(candidate),
        Err(WorthQueryPreviewForkDenial::Create(
            WorthQueryProductBranchCreateError::ProductUnpublished(original),
        )) => original,
        Err(other) => return Err(other),
    };
    continue_expected_recovery(home, recovery, scope)
}

impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHome<Schema, Program> {
    /// Forks a new preview branch from the Home's exact current active head,
    /// completing Directory Home product publication when needed.
    pub fn fork_preview(
        &self,
        label: impl Into<String>,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        let scope = fresh_preview_scope();
        self.fork_preview_with_scope(label, &scope)
    }

    /// Forks a new preview branch under an explicit [`WorthQueryRequestScope`],
    /// preserving exact recovery custody on pre-entry refusal.
    pub fn fork_preview_with_scope(
        &self,
        label: impl Into<String>,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        let label = label.into();
        let expected = fork_expected_from_active_with_scope(self, scope)?;
        Ok(WorthQueryPreviewFork { label, expected })
    }

    /// Continues an unpublished preview creation recovery under `scope`,
    /// preserving exact recovery custody if pre-entry admission refuses.
    pub fn continue_preview_recovery(
        &self,
        label: impl Into<String>,
        recovery: WorthQueryProductBranchCreationRecovery,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        let label = label.into();
        let expected = continue_expected_recovery(self, recovery, scope)?;
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
    ///
    /// Refuses with [`WorthQueryPreviewDiscardDenial::ActiveBranchProtected`]
    /// if `branch` is currently this Home's active head, coordinated atomically
    /// with [`Self::adopt_preview`].
    pub fn close_branch(
        &self,
        branch: WorthQueryProductBranch,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryPreviewDiscardDenial> {
        let head = self
            .active_head
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *head == branch {
            return Err(WorthQueryPreviewDiscardDenial::ActiveBranchProtected);
        }
        let scope = fresh_preview_scope();
        let result = self
            .primary_graph()
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
            .map_err(WorthQueryPreviewDiscardDenial::Advancement)?;
        drop(head);
        result
    }
}
