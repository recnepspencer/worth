use super::PlatformPulseLifecycleObservationProjectionDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::observation_contract) enum PlatformPulseVisualObservationState {
    AwaitingFirstFrame,
    AwaitingSnapshot {
        frame: u64,
    },
    SnapshotCaptured {
        snapshot: u64,
        frame: u64,
    },
    IdentityTraced {
        snapshot: u64,
        frame: u64,
        target_receipt: u64,
    },
    OverlayPublished {
        snapshot: u64,
        snapshot_frame: u64,
        overlay: u64,
        published_frame: u64,
    },
    OverlayCleared {
        snapshot: u64,
        snapshot_frame: u64,
        overlay: u64,
        published_frame: u64,
        cleared_frame: u64,
    },
    AwaitingRefreshRetirement {
        snapshot: u64,
        snapshot_frame: u64,
        refresh_frame: u64,
    },
    AwaitingRefreshSnapshot {
        refresh_frame: u64,
    },
    Refreshed {
        snapshot: u64,
        frame: u64,
    },
    AwaitingSuccessorSnapshot {
        predecessor_snapshot: u64,
        predecessor_frame: u64,
        successor_frame: u64,
    },
    AwaitingComparison {
        predecessor_snapshot: u64,
        predecessor_frame: u64,
        successor_snapshot: u64,
        successor_frame: u64,
    },
    AwaitingRetirement {
        snapshot: u64,
        snapshot_frame: u64,
        successor_frame: u64,
    },
    Retired,
}

impl PlatformPulseVisualObservationState {
    pub(in crate::observation_contract) fn after_refreshed_snapshot(
        self,
        snapshot: u64,
        observed_frame: u64,
        observed_current: bool,
    ) -> Result<Self, PlatformPulseLifecycleObservationProjectionDenial> {
        let Self::AwaitingRefreshSnapshot { refresh_frame } = self else {
            return Err(
                PlatformPulseLifecycleObservationProjectionDenial::VisualObservationOutOfOrder,
            );
        };
        if !observed_current || observed_frame < refresh_frame {
            return Err(PlatformPulseLifecycleObservationProjectionDenial::
                VisualRefreshSnapshotAffinityMismatch {
                    expected_frame: refresh_frame,
                    observed_frame,
                    observed_current,
                });
        }
        Ok(Self::Refreshed {
            snapshot,
            frame: observed_frame,
        })
    }

    pub(in crate::observation_contract) fn after_content_publication(
        self,
        frame: u64,
    ) -> Result<Self, PlatformPulseLifecycleObservationProjectionDenial> {
        match self {
            Self::AwaitingSnapshot { frame: previous } if frame >= previous => {
                Ok(Self::AwaitingSnapshot { frame })
            }
            // The live overlay still owns its exact publication and must clear
            // before its retained snapshot can be retired and refreshed.
            Self::OverlayPublished {
                published_frame, ..
            } if frame >= published_frame => Ok(self),
            Self::Retired => Ok(Self::AwaitingRefreshSnapshot {
                refresh_frame: frame,
            }),
            Self::OverlayCleared {
                snapshot,
                snapshot_frame,
                ..
            }
            | Self::Refreshed {
                snapshot,
                frame: snapshot_frame,
            } => Ok(Self::AwaitingRefreshRetirement {
                snapshot,
                snapshot_frame,
                refresh_frame: frame,
            }),
            Self::AwaitingRefreshRetirement {
                snapshot,
                snapshot_frame,
                refresh_frame,
            } if frame >= refresh_frame => Ok(Self::AwaitingRefreshRetirement {
                snapshot,
                snapshot_frame,
                refresh_frame: frame,
            }),
            Self::AwaitingRefreshSnapshot { refresh_frame } if frame >= refresh_frame => {
                Ok(Self::AwaitingRefreshSnapshot {
                    refresh_frame: frame,
                })
            }
            Self::AwaitingSuccessorSnapshot {
                predecessor_snapshot,
                predecessor_frame,
                successor_frame,
            } if frame >= successor_frame => Ok(Self::AwaitingRefreshRetirement {
                snapshot: predecessor_snapshot,
                snapshot_frame: predecessor_frame,
                refresh_frame: frame,
            }),
            _ => Err(PlatformPulseLifecycleObservationProjectionDenial::VisualPulseIncomplete),
        }
    }

    pub(in crate::observation_contract) fn after_replacement(
        self,
        successor_frame: u64,
    ) -> Result<Self, PlatformPulseLifecycleObservationProjectionDenial> {
        match self {
            Self::AwaitingSnapshot { .. } => Ok(Self::AwaitingSnapshot {
                frame: successor_frame,
            }),
            Self::OverlayCleared {
                snapshot,
                snapshot_frame,
                ..
            } => Ok(Self::AwaitingSuccessorSnapshot {
                predecessor_snapshot: snapshot,
                predecessor_frame: snapshot_frame,
                successor_frame,
            }),
            Self::Refreshed { snapshot, frame } => Ok(Self::AwaitingSuccessorSnapshot {
                predecessor_snapshot: snapshot,
                predecessor_frame: frame,
                successor_frame,
            }),
            Self::Retired => Ok(Self::AwaitingRefreshSnapshot {
                refresh_frame: successor_frame,
            }),
            Self::AwaitingFirstFrame
            | Self::SnapshotCaptured { .. }
            | Self::IdentityTraced { .. }
            | Self::OverlayPublished { .. }
            | Self::AwaitingRefreshRetirement { .. }
            | Self::AwaitingRefreshSnapshot { .. }
            | Self::AwaitingSuccessorSnapshot { .. }
            | Self::AwaitingComparison { .. }
            | Self::AwaitingRetirement { .. } => {
                Err(PlatformPulseLifecycleObservationProjectionDenial::VisualPulseIncomplete)
            }
        }
    }
}
