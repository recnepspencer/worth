use worth_proof::CanonicalVec;
use worth_store_physical_format::{PhysicalRewriteRedo, RecordArtifactFile};
use worth_store_wal::WalLsnRange;

pub(in crate::physical_runtime) use super::prepared_frames::PhysicalDataPlanBindingDenial;
pub(in crate::physical_runtime) use super::prepared_frames::{
    PreparedPhysicalDataFrame, WalBoundPhysicalDataFrame,
};
use super::{
    prepared_frames::{PreparedFrameDataPlan, WalBoundFrameDataPlan},
    PhysicalRedoTargetClaim,
};
use crate::physical_runtime::record_serving::AdoptedExtentCopy;

/// Copy adoption and a terminal head retired member are separate typed
/// lanes, never an empty frame plan.
pub(in crate::physical_runtime) enum PreparedPhysicalDataPlan {
    Frames(PreparedFrameDataPlan),
    SourceCopy(AdoptedExtentCopy),
    /// No data frame and no data record: the WAL member is its typed recovery
    /// operation and the root publication is its only effect.
    TerminalHeadRetirement,
}

pub(in crate::physical_runtime) enum WalBoundPhysicalDataPlan {
    Frames(WalBoundFrameDataPlan),
    SourceCopy {
        copy: AdoptedExtentCopy,
        publication: WalLsnRange,
    },
    TerminalHeadRetirement {
        publication: WalLsnRange,
    },
}

impl PreparedPhysicalDataPlan {
    pub(in crate::physical_runtime) fn source_copy(&self) -> Option<&AdoptedExtentCopy> {
        match self {
            Self::SourceCopy(copy) => Some(copy),
            Self::Frames(_) | Self::TerminalHeadRetirement => None,
        }
    }
    pub(in crate::physical_runtime) fn new(
        frames: Vec<PreparedPhysicalDataFrame>,
        count: u32,
    ) -> Result<Self, PhysicalDataPlanBindingDenial> {
        PreparedFrameDataPlan::new(frames, count).map(Self::Frames)
    }

    pub(in crate::physical_runtime) fn with_rewrite(self, rewrite: PhysicalRewriteRedo) -> Self {
        match self {
            Self::Frames(plan) => Self::Frames(plan.with_rewrite(rewrite)),
            Self::SourceCopy(_) => unreachable!("copy adoption has its own typed redo recipe"),
            Self::TerminalHeadRetirement => {
                unreachable!("a terminal head retired member rewrites no page")
            }
        }
    }

    pub(in crate::physical_runtime) fn retained_growth(&self) -> Vec<(RecordArtifactFile, u64)> {
        match self {
            Self::Frames(plan) => plan.retained_growth(),
            // The durable copy intent owns its already-written destination charge.
            Self::SourceCopy(_) => Vec::new(),
            Self::TerminalHeadRetirement => Vec::new(),
        }
    }

    pub(in crate::physical_runtime) fn bind(
        self,
        range: WalLsnRange,
    ) -> Result<WalBoundPhysicalDataPlan, (Self, PhysicalDataPlanBindingDenial)> {
        match self {
            Self::Frames(plan) => plan
                .bind(range)
                .map(WalBoundPhysicalDataPlan::Frames)
                .map_err(|(plan, error)| (Self::Frames(plan), error)),
            Self::SourceCopy(copy) => {
                if range.start().get() <= copy.durable_intent_lsn()
                    || range.start().get().checked_add(1) != Some(range.end_exclusive().get())
                {
                    return Err((
                        Self::SourceCopy(copy),
                        PhysicalDataPlanBindingDenial::InvalidWalBasis,
                    ));
                }
                if copy.bind_publication(range.start().get()).is_err() {
                    return Err((
                        Self::SourceCopy(copy),
                        PhysicalDataPlanBindingDenial::InvalidWalBasis,
                    ));
                }
                Ok(WalBoundPhysicalDataPlan::SourceCopy {
                    copy,
                    publication: range,
                })
            }
            // A record-less member still occupies exactly one LSN.
            Self::TerminalHeadRetirement => {
                if range.start().get().checked_add(1) != Some(range.end_exclusive().get()) {
                    return Err((
                        Self::TerminalHeadRetirement,
                        PhysicalDataPlanBindingDenial::InvalidWalBasis,
                    ));
                }
                Ok(WalBoundPhysicalDataPlan::TerminalHeadRetirement { publication: range })
            }
        }
    }
}

impl WalBoundPhysicalDataPlan {
    pub(in crate::physical_runtime) fn redo_targets(
        &self,
    ) -> Option<&[CanonicalVec<PhysicalRedoTargetClaim>]> {
        match self {
            Self::Frames(plan) => Some(plan.redo_targets()),
            Self::SourceCopy { .. } | Self::TerminalHeadRetirement { .. } => None,
        }
    }
    pub(in crate::physical_runtime) fn frames(&self) -> Option<&[WalBoundPhysicalDataFrame]> {
        match self {
            Self::Frames(plan) => Some(plan.frames()),
            Self::SourceCopy { .. } | Self::TerminalHeadRetirement { .. } => None,
        }
    }
    pub(in crate::physical_runtime) const fn rewrite(&self) -> Option<PhysicalRewriteRedo> {
        match self {
            Self::Frames(plan) => plan.rewrite(),
            Self::SourceCopy { .. } | Self::TerminalHeadRetirement { .. } => None,
        }
    }
    pub(in crate::physical_runtime) fn source_copy(
        &self,
    ) -> Option<(&AdoptedExtentCopy, WalLsnRange)> {
        match self {
            Self::SourceCopy { copy, publication } => Some((copy, *publication)),
            Self::Frames(_) | Self::TerminalHeadRetirement { .. } => None,
        }
    }
    /// The one LSN a terminal head retired member occupies.
    pub(in crate::physical_runtime) const fn terminal_head_retirement(
        &self,
    ) -> Option<WalLsnRange> {
        match self {
            Self::TerminalHeadRetirement { publication } => Some(*publication),
            Self::Frames(_) | Self::SourceCopy { .. } => None,
        }
    }
    pub(in crate::physical_runtime) fn into_prepared(self) -> PreparedPhysicalDataPlan {
        match self {
            Self::Frames(plan) => PreparedPhysicalDataPlan::Frames(plan.into_prepared()),
            Self::SourceCopy { copy, publication } => {
                copy.release_publication_before_effect(publication.start().get())
                    .expect("restoring a WAL reservation proves no publication effect escaped");
                PreparedPhysicalDataPlan::SourceCopy(copy)
            }
            Self::TerminalHeadRetirement { .. } => PreparedPhysicalDataPlan::TerminalHeadRetirement,
        }
    }
}
