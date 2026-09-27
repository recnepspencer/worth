use crate::runtime::allocation_frame_dispatch::{
    UiAllocationFrameEpoch, UiAllocationFrameIngressKey, UiAllocationFrameIngressSequence,
    UiAllocationFrameMailboxDrain,
};

use super::super::dispatcher::UiAllocationFrameSealAuthority;
use super::UiAllocationFrameDispatcherCounters;

/// Immutable, canonically ordered dispatcher output reserved for the later runtime turn.
///
/// The handoff is deliberately move-only.
///
/// ```compile_fail
/// use worth_ui_runtime::facade::runtime_handoff::UiAdmittedAllocationStreamFrame;
///
/// fn cannot_duplicate(frame: UiAdmittedAllocationStreamFrame) {
///     let _duplicate = frame.clone();
/// }
/// ```
#[derive(Debug, PartialEq)]
pub(crate) struct UiAdmittedAllocationStreamFrame {
    epoch: UiAllocationFrameEpoch,
    ingress: UiAllocationFrameMailboxDrain,
    assignments: UiAllocationFrameSubmissionAssignmentBatch,
    counters: UiAllocationFrameDispatcherCounters,
    duplicate_witness: UiAllocationFrameDuplicateWitness,
}

/// Dispatcher-minted proof that the sealed ingress set is canonical and duplicate-free.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationFrameDuplicateWitness {
    canonical_ingress_keys: Box<[UiAllocationFrameIngressKey]>,
    duplicate_count_at_seal: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::runtime::allocation_frame_dispatch) struct UiAllocationFrameSubmissionAssignmentBatch
{
    sequences: [UiAllocationFrameIngressSequence; super::super::ALLOCATION_FRAME_SOURCE_CAPACITY],
}

/// Canonical epoch/sequence proof minted while the dispatcher seals a frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationFrameSubmissionAssignment {
    ingress_key: UiAllocationFrameIngressKey,
    epoch: UiAllocationFrameEpoch,
    sequence: UiAllocationFrameIngressSequence,
}

impl UiAdmittedAllocationStreamFrame {
    pub(in crate::runtime::allocation_frame_dispatch) fn frame_epoch_assignment(
        &self,
    ) -> super::super::UiAllocationFrameEpochAssignment {
        super::super::UiAllocationFrameEpochAssignment::from_sealed_frame(self)
    }

    pub(crate) fn into_policy_input(
        self,
    ) -> (
        UiAllocationFrameEpoch,
        UiAllocationFrameMailboxDrain,
        UiAllocationFrameDuplicateWitness,
    ) {
        (self.epoch, self.ingress, self.duplicate_witness)
    }
    pub(in crate::runtime::allocation_frame_dispatch) fn new(
        _seal_authority: &UiAllocationFrameSealAuthority,
        epoch: UiAllocationFrameEpoch,
        ingress: UiAllocationFrameMailboxDrain,
        assignments: UiAllocationFrameSubmissionAssignmentBatch,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        let duplicate_witness = UiAllocationFrameDuplicateWitness {
            canonical_ingress_keys: ingress.iter().map(|entry| entry.key()).collect(),
            duplicate_count_at_seal: counters.duplicate_count(),
        };
        Self {
            epoch,
            ingress,
            assignments,
            counters,
            duplicate_witness,
        }
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn epoch(&self) -> UiAllocationFrameEpoch {
        self.epoch
    }
    #[cfg(test)]
    pub fn ingress(&self) -> super::super::mailbox::UiAllocationFrameIngressView<'_> {
        self.ingress.view()
    }
    #[cfg(test)]
    pub(crate) fn counters(&self) -> UiAllocationFrameDispatcherCounters {
        self.counters
    }

    #[cfg(test)]
    pub(crate) fn submission_assignments(
        &self,
    ) -> impl ExactSizeIterator<Item = UiAllocationFrameSubmissionAssignment> + '_ {
        self.assignments.iter(self.epoch, &self.ingress)
    }
}

impl UiAllocationFrameDuplicateWitness {
    pub fn canonical_ingress_keys(&self) -> &[UiAllocationFrameIngressKey] {
        &self.canonical_ingress_keys
    }

    pub fn duplicate_count_at_seal(&self) -> u64 {
        self.duplicate_count_at_seal
    }
}

impl UiAllocationFrameSubmissionAssignmentBatch {
    pub(in crate::runtime::allocation_frame_dispatch) fn linearize(
        authority: &UiAllocationFrameSealAuthority,
        epoch: UiAllocationFrameEpoch,
        ingress: &UiAllocationFrameMailboxDrain,
    ) -> Self {
        let mut sequences = [UiAllocationFrameIngressSequence::assign(authority, epoch, 0);
            super::super::ALLOCATION_FRAME_SOURCE_CAPACITY];
        for (ordinal, sequence) in
            (1..=u16::MAX).zip(sequences.iter_mut().take(ingress.iter().len()))
        {
            *sequence = UiAllocationFrameIngressSequence::assign(authority, epoch, ordinal);
        }
        Self { sequences }
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn iter<'a>(
        &'a self,
        epoch: UiAllocationFrameEpoch,
        ingress: &'a UiAllocationFrameMailboxDrain,
    ) -> impl ExactSizeIterator<Item = UiAllocationFrameSubmissionAssignment> + 'a {
        ingress
            .iter()
            .zip(self.sequences.iter().copied())
            .map(
                move |(ingress, sequence)| UiAllocationFrameSubmissionAssignment {
                    ingress_key: ingress.key(),
                    epoch,
                    sequence,
                },
            )
    }
}

impl UiAllocationFrameSubmissionAssignment {
    pub fn ingress_key(&self) -> UiAllocationFrameIngressKey {
        self.ingress_key.clone()
    }
    #[cfg(test)]
    pub fn epoch(&self) -> UiAllocationFrameEpoch {
        self.epoch
    }
    pub fn sequence(&self) -> UiAllocationFrameIngressSequence {
        self.sequence
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct UiAllocationFrameTransitionOutcome {
    representation: UiAllocationFrameTransitionRepresentation,
}

#[derive(Debug, PartialEq)]
enum UiAllocationFrameTransitionRepresentation {
    Dispatched(Box<UiAdmittedAllocationStreamFrame>),
    Denied {
        denial: UiAllocationFrameDispatchDenial,
        counters: UiAllocationFrameDispatcherCounters,
    },
}

impl UiAllocationFrameTransitionOutcome {
    pub(in crate::runtime::allocation_frame_dispatch) fn dispatched(
        _seal_authority: &UiAllocationFrameSealAuthority,
        frame: UiAdmittedAllocationStreamFrame,
    ) -> Self {
        Self {
            representation: UiAllocationFrameTransitionRepresentation::Dispatched(Box::new(frame)),
        }
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn denied(
        _seal_authority: &UiAllocationFrameSealAuthority,
        denial: UiAllocationFrameDispatchDenial,
        counters: UiAllocationFrameDispatcherCounters,
    ) -> Self {
        Self {
            representation: UiAllocationFrameTransitionRepresentation::Denied { denial, counters },
        }
    }

    #[cfg(test)]
    pub(crate) fn dispatched_frame(&self) -> Option<&UiAdmittedAllocationStreamFrame> {
        match &self.representation {
            UiAllocationFrameTransitionRepresentation::Dispatched(frame) => Some(frame),
            UiAllocationFrameTransitionRepresentation::Denied { .. } => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn denial(&self) -> Option<UiAllocationFrameDispatchDenial> {
        match self.representation {
            UiAllocationFrameTransitionRepresentation::Dispatched(_) => None,
            UiAllocationFrameTransitionRepresentation::Denied { denial, .. } => Some(denial),
        }
    }

    #[cfg(test)]
    pub(crate) fn counters(&self) -> UiAllocationFrameDispatcherCounters {
        match &self.representation {
            UiAllocationFrameTransitionRepresentation::Dispatched(frame) => frame.counters(),
            UiAllocationFrameTransitionRepresentation::Denied { counters, .. } => *counters,
        }
    }

    pub(crate) fn into_dispatched_frame(
        self,
    ) -> Result<UiAdmittedAllocationStreamFrame, UiAllocationFrameDispatchDenial> {
        match self.representation {
            UiAllocationFrameTransitionRepresentation::Dispatched(frame) => Ok(*frame),
            UiAllocationFrameTransitionRepresentation::Denied { denial, .. } => Err(denial),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationFrameDispatchDenial {
    NoOpenFrame,
    EmptyFrame,
    EpochExhausted,
    ReplacementNotQuiescent,
}
