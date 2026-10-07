use std::mem::size_of;

use worth_foundational::{ExecutionReport, PartitionIdentity};
use worth_proof::CanonicalUniqueVec;

use crate::{
    authority::ExecutionResourceLease,
    backend::{run_ordered, KernelContext, KernelFailure, OrderedOutcome},
    report::ChargedBytes,
};

use super::MapStop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDenial {
    IdentitiesNotCanonical,
    CoverageMismatch,
    MemoryOverflow,
}

/// Ordered items and identities admitted before the first carry is evaluated.
pub struct ExecutionScan<T> {
    identities: CanonicalUniqueVec<PartitionIdentity>,
    items: Vec<T>,
    input_bytes: u64,
}

pub enum ScanOutcome<S, O, E> {
    Complete {
        state: S,
        prefixes: Vec<O>,
        report: ExecutionReport,
    },
    Stopped {
        completed_state: S,
        completed_prefix: Vec<O>,
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
    },
}

impl<T: ChargedBytes> ExecutionScan<T> {
    pub fn try_from_ordered(
        expected_identities: Vec<PartitionIdentity>,
        entries: Vec<(PartitionIdentity, T)>,
    ) -> Result<Self, ScanDenial> {
        let identity_bytes = expected_identities
            .capacity()
            .checked_mul(size_of::<PartitionIdentity>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(ScanDenial::MemoryOverflow)?;
        let identities = CanonicalUniqueVec::try_from_sorted_unique(expected_identities)
            .map_err(|_| ScanDenial::IdentitiesNotCanonical)?;
        if !identities
            .as_slice()
            .iter()
            .copied()
            .eq(entries.iter().map(|(identity, _)| *identity))
        {
            return Err(ScanDenial::CoverageMismatch);
        }
        let items: Vec<_> = entries.into_iter().map(|(_, value)| value).collect();
        let input_bytes = items
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| {
                items.iter().try_fold(bytes, |sum, item| {
                    sum.checked_add(item.additional_charged_bytes())
                })
            })
            .and_then(|bytes| bytes.checked_add(identity_bytes))
            .ok_or(ScanDenial::MemoryOverflow)?;
        Ok(Self {
            identities,
            items,
            input_bytes,
        })
    }

    pub fn run<S, O, E, F>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        initial: S,
        max_state_bytes: u64,
        max_prefix_bytes: u64,
        max_error_bytes: u64,
        scratch_bytes: u64,
        mut step: F,
    ) -> ScanOutcome<S, O, E>
    where
        S: ChargedBytes,
        O: ChargedBytes,
        E: ChargedBytes,
        F: FnMut(&S, &T, &mut KernelContext<'_, '_>) -> Result<(S, O), KernelFailure<E>>,
    {
        let result = run_ordered(
            lease,
            &self.identities,
            initial,
            max_state_bytes,
            max_prefix_bytes,
            max_error_bytes,
            scratch_bytes,
            self.input_bytes,
            |state, index, context| step(state, &self.items[index], context),
        );
        result.into()
    }
}

impl<S, O, E> From<OrderedOutcome<S, O, E>> for ScanOutcome<S, O, E> {
    fn from(outcome: OrderedOutcome<S, O, E>) -> Self {
        match outcome.stop {
            None => Self::Complete {
                state: outcome.state,
                prefixes: outcome.outputs,
                report: outcome.report,
            },
            Some(reason) => Self::Stopped {
                completed_state: outcome.state,
                completed_prefix: outcome.outputs,
                boundary: outcome.boundary,
                reason,
                report: outcome.report,
            },
        }
    }
}

impl<S, O, E> ScanOutcome<S, O, E> {
    pub fn report(&self) -> ExecutionReport {
        match self {
            Self::Complete { report, .. } | Self::Stopped { report, .. } => *report,
        }
    }
}
