use std::{mem::size_of, num::NonZeroUsize};

use worth_foundational::{ExecutionReport, PartitionIdentity};
use worth_proof::CanonicalUniqueVec;

use crate::{
    authority::ExecutionResourceLease,
    backend::{run_ordered_until, KernelContext, KernelFailure, OrderedStep},
    report::ChargedBytes,
};

use super::MapStop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundsDenial {
    RoundCountOverflow,
    MemoryUnavailable,
}

/// A declared finite sequence of barriers. Every step observes one immutable
/// committed image; its replacement becomes visible only after that step and
/// its convergence decision both succeed.
pub struct ExecutionRounds {
    identities: CanonicalUniqueVec<PartitionIdentity>,
    identity_bytes: u64,
}

pub enum RoundsOutcome<S, E> {
    Converged {
        state: S,
        rounds: usize,
        report: ExecutionReport,
    },
    NotConverged {
        state: S,
        rounds: usize,
        report: ExecutionReport,
    },
    Stopped {
        completed_state: S,
        completed_rounds: usize,
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
    },
}

impl ExecutionRounds {
    pub fn try_new(max_rounds: NonZeroUsize) -> Result<Self, RoundsDenial> {
        let count = max_rounds.get();
        u64::try_from(count).map_err(|_| RoundsDenial::RoundCountOverflow)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| RoundsDenial::MemoryUnavailable)?;
        for round in 1..=count {
            values.push(PartitionIdentity::new(round as u64));
        }
        let identity_bytes = values
            .capacity()
            .checked_mul(size_of::<PartitionIdentity>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(RoundsDenial::RoundCountOverflow)?;
        let identities = CanonicalUniqueVec::try_from_sorted_unique(values)
            .expect("round ordinals are canonical");
        Ok(Self {
            identities,
            identity_bytes,
        })
    }

    pub fn max_rounds(&self) -> usize {
        self.identities.as_slice().len()
    }

    pub fn run<S, E, F, C>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        initial: S,
        max_state_bytes: u64,
        max_error_bytes: u64,
        scratch_bytes: u64,
        mut step: F,
        mut converged: C,
    ) -> RoundsOutcome<S, E>
    where
        S: ChargedBytes,
        E: ChargedBytes,
        F: FnMut(&S, usize, &mut KernelContext<'_, '_>) -> Result<S, KernelFailure<E>>,
        C: FnMut(&S, &S) -> bool,
    {
        let result = run_ordered_until(
            lease,
            &self.identities,
            initial,
            max_state_bytes,
            0,
            max_error_bytes,
            scratch_bytes,
            self.identity_bytes,
            |prior, index, context| {
                let next = step(prior, index + 1, context)?;
                if converged(prior, &next) {
                    Ok(OrderedStep::Complete(next, ()))
                } else {
                    Ok(OrderedStep::Continue(next, ()))
                }
            },
        );
        let completed_rounds = result.outputs.len();
        match result.stop {
            Some(reason) => RoundsOutcome::Stopped {
                completed_state: result.state,
                completed_rounds,
                boundary: result.boundary,
                reason,
                report: result.report,
            },
            None if result.completed_early => RoundsOutcome::Converged {
                state: result.state,
                rounds: completed_rounds,
                report: result.report,
            },
            None => RoundsOutcome::NotConverged {
                state: result.state,
                rounds: completed_rounds,
                report: result.report,
            },
        }
    }
}

impl<S, E> RoundsOutcome<S, E> {
    pub fn report(&self) -> ExecutionReport {
        match self {
            Self::Converged { report, .. }
            | Self::NotConverged { report, .. }
            | Self::Stopped { report, .. } => *report,
        }
    }
}
