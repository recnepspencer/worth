//! Infallible field moves keep every acquired resource in its current phase.
use super::{phase_loan::OriginalContinuationPhaseLoan, *};

impl OriginalContinuationPhaseLoan<'_> {
    pub(super) fn submit(&mut self, permit: crate::branch::WorldProductMovementPermit) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Prepared(work) => {
                OriginalContinuationProgress::Submitted(SubmittedContinuationWork { work, permit })
            }
            OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn acknowledge(&mut self) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Submitted(work) => {
                OriginalContinuationProgress::Acknowledged(work)
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn unsubmit(&mut self) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Submitted(work) => {
                work.permit.proven_no_effect();
                OriginalContinuationProgress::Prepared(work.work)
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn materialize(&mut self, occurrence: crate::durability::AdmittedWorldOccurrence) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Acknowledged(work) => {
                OriginalContinuationProgress::Materialized(work.materialize(occurrence))
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn install_history(
        &mut self,
        history_head: crate::history::ProductHeadHistoryProtectionObligation,
        publication: WorldReadmittedPublication,
    ) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Materialized(work) => {
                OriginalContinuationProgress::HistoryInstalled(HistoryInstalledContinuationWork {
                    common: work.common,
                    record: work.record,
                    permit: work.permit,
                    history_head,
                    publication,
                })
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn transfer(&mut self, transfer: crate::retention::ProductHeadRetentionTransfer) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::HistoryInstalled(work) => {
                OriginalContinuationProgress::Transferred(TransferredContinuationWork {
                    common: work.common,
                    record: work.record,
                    permit: work.permit,
                    history_head: work.history_head,
                    publication: work.publication,
                    transfer,
                })
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn install_head(&mut self) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Transferred(work) => {
                let head = crate::branch::ProductBranchHeadProtection::owner_issued(
                    work.common.after.clone(),
                    work.transfer,
                    work.history_head,
                );
                let (head, admitted) = match head {
                    Ok(head) => (head, true),
                    Err(cause) => (cause.into_protection(), false),
                };
                let installed = InstalledContinuationWork {
                    common: work.common,
                    record: work.record,
                    permit: Some(work.permit),
                    head: Some(head),
                    publication: Some(work.publication),
                };
                if admitted {
                    OriginalContinuationProgress::Installed(installed)
                } else {
                    OriginalContinuationProgress::ReopenRequired(installed)
                }
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn prearm_reopen(&mut self) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Installed(work) => {
                OriginalContinuationProgress::ReopenRequired(work)
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn complete(&mut self) {
        self.transition(|phase| match phase {
            OriginalContinuationProgress::ReopenRequired(mut work) => {
                OriginalContinuationProgress::Completed(
                    work.publication
                        .take()
                        .expect("actual history caller protection was installed before the head"),
                )
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::Completed(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("the admitted continuation phase owns this move")
            }
        });
    }
    pub(super) fn deliver(&mut self) -> OriginalContinuationAdvance {
        let mut delivered = None;
        self.transition(|phase| match phase {
            OriginalContinuationProgress::Completed(publication) => {
                delivered = Some(publication);
                OriginalContinuationProgress::Delivered
            }
            OriginalContinuationProgress::Prepared(_)
            | OriginalContinuationProgress::Submitted(_)
            | OriginalContinuationProgress::Acknowledged(_)
            | OriginalContinuationProgress::Materialized(_)
            | OriginalContinuationProgress::HistoryInstalled(_)
            | OriginalContinuationProgress::Transferred(_)
            | OriginalContinuationProgress::Installed(_)
            | OriginalContinuationProgress::ReopenRequired(_)
            | OriginalContinuationProgress::Delivered => {
                unreachable!("only a completed original can be delivered")
            }
        });
        OriginalContinuationAdvance::Completed(delivered.expect("infallible caller transfer"))
    }
}
