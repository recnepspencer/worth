//! Whether a row still owes work, keeps an answer, or can retire.
use super::{DemandRecord, DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint};

pub(super) enum IdleRowDisposition {
    InFlight,
    KeptAnswer,
    HeldForStaleOwner,
    Retired(RowRetirement),
}

#[derive(Clone, Copy)]
pub(super) enum RowRetirement {
    Unstarted,
    Finished,
    AbandonedPublication,
}

impl DemandRecord {
    /// Checkpoints still owned by an interest await that owner's entry. Once
    /// its last interest and last awaiting stale owner close, an undelivered
    /// checkpoint answers nobody. The caller supplies the occurrence owner's
    /// awaited fact; claims, cleanup and performed-source holds are separate.
    pub(super) fn idle_disposition(&self, awaited_by_stale_owner: bool) -> IdleRowDisposition {
        use IdleRowDisposition::{InFlight, KeptAnswer, Retired};
        match &self.state {
            DemandState::Scheduling | DemandState::Running => InFlight,
            DemandState::Admitted | DemandState::Scheduled => {
                if self.interests == 0 {
                    Retired(RowRetirement::Unstarted)
                } else {
                    InFlight
                }
            }
            DemandState::Failed(_) => Retired(RowRetirement::Finished),
            DemandState::Output(output) => match output.advancement() {
                WorthQueryOutputAdvancement::Claimed(_) => InFlight,
                WorthQueryOutputAdvancement::Stopped { .. } => Retired(RowRetirement::Finished),
                WorthQueryOutputAdvancement::Idle => match &output.checkpoint {
                    Some(WorthQueryOutputCheckpoint::Ready(_)) => KeptAnswer,
                    Some(
                        WorthQueryOutputCheckpoint::Published { .. }
                        | WorthQueryOutputCheckpoint::Delivered { .. },
                    ) => {
                        if awaited_by_stale_owner {
                            IdleRowDisposition::HeldForStaleOwner
                        } else if self.interests == 0 {
                            Retired(RowRetirement::AbandonedPublication)
                        } else {
                            InFlight
                        }
                    }
                    None => {
                        if self.interests == 0 {
                            Retired(RowRetirement::Unstarted)
                        } else {
                            InFlight
                        }
                    }
                },
            },
        }
    }
}

impl RowRetirement {
    /// An unstarted row still owes any performed operation that holds it.
    pub(super) fn releases_finished_work(self) -> bool {
        match self {
            Self::Unstarted => false,
            Self::Finished | Self::AbandonedPublication => true,
        }
    }
}

impl DemandRecord {
    pub(super) fn terminal(&self) -> bool {
        matches!(
            self.idle_disposition(false),
            IdleRowDisposition::Retired(RowRetirement::Finished)
        )
    }
}
