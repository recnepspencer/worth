use crate::runtime::allocation_frame_dispatch::{
    UiAllocationFrameEpoch, UiAllocationFrameIngressIdentity, UiAllocationFrameIngressKey,
    UiAllocationFrameIngressSequence, UiAllocationFrameSubmissionDenial,
};

use super::super::dispatcher::UiAllocationFrameSealAuthority;
use super::UiAllocationFrameDispatcherCounters;

/// Self-describing result issued only by the allocation-frame linearizer.
#[derive(Debug, Eq, PartialEq)]
pub struct UiAllocationFrameSubmissionOutcome {
    ingress_key: Box<UiAllocationFrameIngressKey>,
    counters: Box<UiAllocationFrameDispatcherCounters>,
    representation: UiAllocationFrameSubmissionRepresentation,
}

#[derive(Debug, Eq, PartialEq)]
enum UiAllocationFrameSubmissionRepresentation {
    Queued {
        epoch: UiAllocationFrameEpoch,
    },
    DuplicatePending {
        epoch: UiAllocationFrameEpoch,
    },
    DuplicateAssigned {
        epoch: UiAllocationFrameEpoch,
        sequence: UiAllocationFrameIngressSequence,
    },
    LateIngress {
        retry_epoch: UiAllocationFrameEpoch,
    },
    Backpressured {
        watermark: u16,
        retry_epoch: UiAllocationFrameEpoch,
    },
    Denied(UiAllocationFrameSubmissionDenial),
}

impl UiAllocationFrameSubmissionOutcome {
    pub(in crate::runtime::allocation_frame_dispatch) fn queued(
        _seal_authority: &UiAllocationFrameSealAuthority,
        ingress_key: UiAllocationFrameIngressKey,
        epoch: UiAllocationFrameEpoch,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self::new(
            ingress_key,
            counters,
            UiAllocationFrameSubmissionRepresentation::Queued { epoch },
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn duplicate_pending(
        _seal_authority: &UiAllocationFrameSealAuthority,
        ingress_key: UiAllocationFrameIngressKey,
        epoch: UiAllocationFrameEpoch,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self::new(
            ingress_key,
            counters,
            UiAllocationFrameSubmissionRepresentation::DuplicatePending { epoch },
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn duplicate_assigned(
        _seal_authority: &UiAllocationFrameSealAuthority,
        ingress_key: UiAllocationFrameIngressKey,
        epoch: UiAllocationFrameEpoch,
        sequence: UiAllocationFrameIngressSequence,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self::new(
            ingress_key,
            counters,
            UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { epoch, sequence },
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn late_ingress(
        _seal_authority: &UiAllocationFrameSealAuthority,
        ingress_key: UiAllocationFrameIngressKey,
        retry_epoch: UiAllocationFrameEpoch,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self::new(
            ingress_key,
            counters,
            UiAllocationFrameSubmissionRepresentation::LateIngress { retry_epoch },
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn backpressured(
        _seal_authority: &UiAllocationFrameSealAuthority,
        ingress_key: UiAllocationFrameIngressKey,
        watermark: u16,
        retry_epoch: UiAllocationFrameEpoch,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self::new(
            ingress_key,
            counters,
            UiAllocationFrameSubmissionRepresentation::Backpressured {
                watermark,
                retry_epoch,
            },
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn denied(
        _seal_authority: &UiAllocationFrameSealAuthority,
        ingress_key: UiAllocationFrameIngressKey,
        denial: UiAllocationFrameSubmissionDenial,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self::new(
            ingress_key,
            counters,
            UiAllocationFrameSubmissionRepresentation::Denied(denial),
        )
    }

    fn new(
        ingress_key: UiAllocationFrameIngressKey,
        counters: UiAllocationFrameDispatcherCounters,
        representation: UiAllocationFrameSubmissionRepresentation,
    ) -> Self {
        Self {
            ingress_key: Box::new(ingress_key),
            counters: Box::new(counters),
            representation,
        }
    }

    pub fn ingress_key(&self) -> UiAllocationFrameIngressKey {
        (*self.ingress_key).clone()
    }

    pub fn ingress_identity(&self) -> UiAllocationFrameIngressIdentity {
        self.ingress_key.ingress_identity()
    }

    pub fn counters(&self) -> UiAllocationFrameDispatcherCounters {
        *self.counters
    }

    pub fn is_queued(&self) -> bool {
        matches!(
            self.representation,
            UiAllocationFrameSubmissionRepresentation::Queued { .. }
                | UiAllocationFrameSubmissionRepresentation::LateIngress { .. }
        )
    }

    pub fn is_duplicate(&self) -> bool {
        matches!(
            self.representation,
            UiAllocationFrameSubmissionRepresentation::DuplicatePending { .. }
                | UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { .. }
        )
    }

    pub fn is_late_ingress(&self) -> bool {
        matches!(
            self.representation,
            UiAllocationFrameSubmissionRepresentation::LateIngress { .. }
        )
    }

    pub fn is_backpressured(&self) -> bool {
        matches!(
            self.representation,
            UiAllocationFrameSubmissionRepresentation::Backpressured { .. }
        )
    }

    pub fn epoch(&self) -> Option<UiAllocationFrameEpoch> {
        match self.representation {
            UiAllocationFrameSubmissionRepresentation::Queued { epoch }
            | UiAllocationFrameSubmissionRepresentation::DuplicatePending { epoch }
            | UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { epoch, .. }
            | UiAllocationFrameSubmissionRepresentation::LateIngress { retry_epoch: epoch } => {
                Some(epoch)
            }
            UiAllocationFrameSubmissionRepresentation::Backpressured { .. }
            | UiAllocationFrameSubmissionRepresentation::Denied(_) => None,
        }
    }

    pub fn sequence(&self) -> Option<UiAllocationFrameIngressSequence> {
        match self.representation {
            UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { sequence, .. } => {
                Some(sequence)
            }
            UiAllocationFrameSubmissionRepresentation::Queued { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicatePending { .. }
            | UiAllocationFrameSubmissionRepresentation::LateIngress { .. }
            | UiAllocationFrameSubmissionRepresentation::Backpressured { .. }
            | UiAllocationFrameSubmissionRepresentation::Denied(_) => None,
        }
    }

    pub fn retry_epoch(&self) -> Option<UiAllocationFrameEpoch> {
        match self.representation {
            UiAllocationFrameSubmissionRepresentation::LateIngress { retry_epoch }
            | UiAllocationFrameSubmissionRepresentation::Backpressured { retry_epoch, .. } => {
                Some(retry_epoch)
            }
            UiAllocationFrameSubmissionRepresentation::Queued { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicatePending { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { .. }
            | UiAllocationFrameSubmissionRepresentation::Denied(_) => None,
        }
    }

    pub fn backpressure_watermark(&self) -> Option<u16> {
        match self.representation {
            UiAllocationFrameSubmissionRepresentation::Backpressured { watermark, .. } => {
                Some(watermark)
            }
            UiAllocationFrameSubmissionRepresentation::Queued { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicatePending { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { .. }
            | UiAllocationFrameSubmissionRepresentation::LateIngress { .. }
            | UiAllocationFrameSubmissionRepresentation::Denied(_) => None,
        }
    }

    pub fn denial(&self) -> Option<UiAllocationFrameSubmissionDenial> {
        match self.representation {
            UiAllocationFrameSubmissionRepresentation::Denied(denial) => Some(denial),
            UiAllocationFrameSubmissionRepresentation::Queued { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicatePending { .. }
            | UiAllocationFrameSubmissionRepresentation::DuplicateAssigned { .. }
            | UiAllocationFrameSubmissionRepresentation::LateIngress { .. }
            | UiAllocationFrameSubmissionRepresentation::Backpressured { .. } => None,
        }
    }
}
