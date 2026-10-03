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
    #[allow(dead_code)] // No live retirement writer yet; selected prefix still models it.
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

    #[allow(dead_code)] // No live retirement writer exists yet; checkpoint fold still models it.
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
