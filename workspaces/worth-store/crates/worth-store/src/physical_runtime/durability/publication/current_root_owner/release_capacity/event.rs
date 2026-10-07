//! One selected WAL transition and its certificate consequence. A Drop cannot
//! exist without the head upsert that its tag-7 Batch must authenticate.

use worth_store_physical_format::ReleaseCustodyHeadMutationV1;

use super::{
    heads::SelectedReleaseHeadStep, ReleaseCertificateCapacityDenial, SelectedReleaseBatchBasis,
    SelectedReleaseCustodyLedger,
};

#[derive(Clone, Copy)]
pub(super) enum PendingReleaseEvent {
    Drop(CheckedDrop),
    Retirement(CheckedRetirement),
}

#[derive(Clone, Copy)]
pub(super) struct CheckedDrop {
    batch: SelectedReleaseBatchBasis,
    head_step: SelectedReleaseHeadStep,
}

#[derive(Clone, Copy)]
pub(super) struct CheckedRetirement {
    head_step: SelectedReleaseHeadStep,
}

impl SelectedReleaseCustodyLedger {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn pending_drop_count(
        &self,
    ) -> usize {
        self.pending_events
            .iter()
            .filter(|event| matches!(**event, PendingReleaseEvent::Drop(_)))
            .count()
    }

    /// Whether the ledger holds released-drop custody that every later
    /// checkpoint must certify: selected heads or transitions still pending.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn holds_release_custody(
        &self,
    ) -> bool {
        !self.pending_events.is_empty()
            || self.checkpoint_heads.len() > 0
            || self.effective_heads.len() > 0
    }

    pub(super) fn pending_last_drop(&self) -> Option<SelectedReleaseBatchBasis> {
        self.pending_events
            .iter()
            .rev()
            .find_map(|event| event.batch())
    }
}

impl PendingReleaseEvent {
    pub(super) fn for_drop(
        batch: SelectedReleaseBatchBasis,
        head_step: SelectedReleaseHeadStep,
    ) -> Result<Self, ReleaseCertificateCapacityDenial> {
        if !matches!(
            head_step.mutation(),
            ReleaseCustodyHeadMutationV1::Upsert { .. }
        ) {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        Ok(Self::Drop(CheckedDrop { batch, head_step }))
    }

    pub(super) fn for_retirement(
        head_step: SelectedReleaseHeadStep,
    ) -> Result<Self, ReleaseCertificateCapacityDenial> {
        if !matches!(head_step.mutation(), ReleaseCustodyHeadMutationV1::RetireTerminal { expected_prior } if expected_prior.terminal())
        {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        Ok(Self::Retirement(CheckedRetirement { head_step }))
    }

    pub(super) fn head_step(self) -> SelectedReleaseHeadStep {
        match self {
            Self::Drop(drop) => drop.head_step,
            Self::Retirement(retirement) => retirement.head_step,
        }
    }

    pub(super) fn batch(self) -> Option<SelectedReleaseBatchBasis> {
        match self {
            Self::Drop(drop) => Some(drop.batch),
            Self::Retirement(_) => None,
        }
    }
}
