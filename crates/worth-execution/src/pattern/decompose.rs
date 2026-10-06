use worth_foundational::{ExecutionReport, PartitionIdentity};
use worth_proof::CanonicalUniqueVec;

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::ScopeStop,
    oracle::CanonicalBits,
    reduction::{ReductionMetrics, ReductionPlan, ReductionRunFailure, ReductionTree},
    report::ChargedBytes,
};

use super::{
    ExecutionMap, MapDenial, MapKernelContext, MapKernelFailure, MapKernelStop, MapOutcome,
    MapPartition, MapStop,
};

mod back;
mod certify;
mod reuse;
mod scope;
mod stages;
use back::admit_back_map;
pub use certify::DecomposeCertificationFailure;
use reuse::{changed_back_identities, same_encoding};

/// Interior result and its contribution to the canonical interface problem.
#[derive(Clone)]
pub struct InteriorResult<I, C> {
    pub interior: I,
    pub contribution: C,
}

impl<I: ChargedBytes, C: ChargedBytes> ChargedBytes for InteriorResult<I, C> {
    fn additional_charged_bytes(&self) -> u64 {
        self.interior
            .additional_charged_bytes()
            .saturating_add(self.contribution.additional_charged_bytes())
    }
}

/// One interface solve returns slices in the admitted partition order.
#[derive(Clone)]
pub struct InterfaceSolution<S, B> {
    pub solution: S,
    pub slices: Vec<B>,
}

impl<S: ChargedBytes, B: ChargedBytes> ChargedBytes for InterfaceSolution<S, B> {
    fn additional_charged_bytes(&self) -> u64 {
        self.solution
            .additional_charged_bytes()
            .saturating_add(self.slices.additional_charged_bytes())
    }
}

/// Input to one admitted back-substitution partition.
pub struct BackInput<I, B> {
    pub interior: I,
    pub interface_slice: B,
}

impl<I: ChargedBytes, B: ChargedBytes> ChargedBytes for BackInput<I, B> {
    fn additional_charged_bytes(&self) -> u64 {
        self.interior
            .additional_charged_bytes()
            .saturating_add(self.interface_slice.additional_charged_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecomposeInputDenial {
    IdentitiesNotCanonical,
    ChangedIdentitiesNotCanonical,
    UnknownIdentity(PartitionIdentity),
    InitialCoverageMismatch,
    InteriorKernelCoverageMismatch,
    InterfaceSliceCoverageMismatch,
    InvalidCanonicalEncoding,
    StagedCapacityExceeded,
    MemoryOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecomposeStage {
    Interior,
    Reduction,
    Interface,
    Back,
}

/// Caller-declared semantic editions for the three callbacks. Bump an edition
/// whenever callback behavior or captured data changes; reuse requires sameness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecomposeKernelEditions {
    pub interior: u64,
    pub interface: u64,
    pub back: u64,
}

#[derive(Debug)]
pub enum DecomposeFailure<E> {
    Input(DecomposeInputDenial),
    Interior {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
    },
    ReductionRun(ReductionRunFailure<MapKernelStop>),
    InterfaceAdmission(MapDenial),
    Interface {
        reason: MapStop<E>,
        report: ExecutionReport,
    },
    BackAdmission(MapDenial),
    Back {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
    },
    ScopeAdmission(LeaseDenial),
    ScopeStopped {
        stage: DecomposeStage,
        reason: MapKernelStop,
    },
    ScopePanic {
        stage: DecomposeStage,
    },
}

impl<E: ChargedBytes> ChargedBytes for DecomposeFailure<E> {
    fn additional_charged_bytes(&self) -> u64 {
        let stop = match self {
            Self::Interior { reason, .. }
            | Self::Interface { reason, .. }
            | Self::Back { reason, .. } => reason,
            _ => return 0,
        };
        match stop {
            MapStop::Failure {
                cause: MapKernelFailure::Domain(error),
                ..
            } => error.additional_charged_bytes(),
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct DecomposeRunFailure<E> {
    pub cause: DecomposeFailure<E>,
    pub total_report: ExecutionReport,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DecomposeReuse {
    pub unchanged_contributions: usize,
    pub interface_solve_reused: bool,
    pub back_substitutions_reused: usize,
}

pub struct DecomposeComplete<O> {
    pub values: Vec<O>,
    pub total_report: ExecutionReport,
    pub interior_report: ExecutionReport,
    pub reduction_metrics: ReductionMetrics,
    pub interface_report: Option<ExecutionReport>,
    pub back_report: Option<ExecutionReport>,
    pub reuse: DecomposeReuse,
}

#[derive(Clone)]
struct Snapshot<I, C, S, B, O, F> {
    editions: DecomposeKernelEditions,
    tree: ReductionTree<C, F>,
    interiors: Vec<I>,
    contributions: Vec<C>,
    assembled: C,
    interface: InterfaceSolution<S, B>,
    outputs: Vec<O>,
}

impl<I: ChargedBytes, C: ChargedBytes, S: ChargedBytes, B: ChargedBytes, O: ChargedBytes, F>
    ChargedBytes for Snapshot<I, C, S, B, O, F>
{
    fn additional_charged_bytes(&self) -> u64 {
        self.tree
            .additional_charged_bytes()
            .saturating_add(self.stage_bytes())
    }
}

impl<I: ChargedBytes, C: ChargedBytes, S: ChargedBytes, B: ChargedBytes, O: ChargedBytes, F>
    Snapshot<I, C, S, B, O, F>
{
    fn stage_bytes(&self) -> u64 {
        self.interiors
            .additional_charged_bytes()
            .saturating_add(self.contributions.additional_charged_bytes())
            .saturating_add(self.assembled.additional_charged_bytes())
            .saturating_add(self.interface.additional_charged_bytes())
            .saturating_add(self.outputs.additional_charged_bytes())
    }
}

struct Staged<I, C, S, B, O, F> {
    snapshot: Snapshot<I, C, S, B, O, F>,
    complete: DecomposeComplete<O>,
}

impl<I: ChargedBytes, C: ChargedBytes, S: ChargedBytes, B: ChargedBytes, O: ChargedBytes, F>
    ChargedBytes for Staged<I, C, S, B, O, F>
{
    fn additional_charged_bytes(&self) -> u64 {
        self.snapshot
            .additional_charged_bytes()
            .saturating_add(self.complete.values.additional_charged_bytes())
    }
}

/// Retained decomposition state. The first run covers every identity; later
/// runs accept a checked map over just the changed identities.
#[derive(Clone)]
pub struct ExecutionDecompose<I, C, S, B, O, F> {
    identities: CanonicalUniqueVec<PartitionIdentity>,
    reduction_identity: C,
    combine: F,
    max_interface_result_bytes: u64,
    max_back_result_bytes: u64,
    max_reduction_value_bytes: u64,
    max_staged_bytes: u64,
    reducer_storage_bytes: u64,
    snapshot: Option<Snapshot<I, C, S, B, O, F>>,
}

impl<I, C, S, B, O, F> ExecutionDecompose<I, C, S, B, O, F>
where
    I: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    C: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    S: Clone + Send + ChargedBytes,
    B: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    O: Clone + Send + ChargedBytes,
    F: Fn(&C, &C) -> C + Clone,
{
    pub fn try_new(
        identities: Vec<PartitionIdentity>,
        reduction_identity: C,
        combine: F,
        max_interface_result_bytes: u64,
        max_back_result_bytes: u64,
        max_reduction_value_bytes: u64,
        max_staged_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> Result<Self, DecomposeInputDenial> {
        let identities = CanonicalUniqueVec::try_from_sorted_unique(identities)
            .map_err(|_| DecomposeInputDenial::IdentitiesNotCanonical)?;
        Ok(Self {
            identities,
            reduction_identity,
            combine,
            max_interface_result_bytes,
            max_back_result_bytes,
            max_reduction_value_bytes,
            max_staged_bytes,
            reducer_storage_bytes,
            snapshot: None,
        })
    }

    fn check_changed(
        &self,
        identities: &[PartitionIdentity],
        editions: DecomposeKernelEditions,
    ) -> Result<(), DecomposeInputDenial> {
        if identities.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(DecomposeInputDenial::ChangedIdentitiesNotCanonical);
        }
        if self.snapshot.is_none() && identities != self.identities.as_slice() {
            return Err(DecomposeInputDenial::InitialCoverageMismatch);
        }
        if self
            .snapshot
            .as_ref()
            .is_some_and(|previous| previous.editions.interior != editions.interior)
            && identities != self.identities.as_slice()
        {
            return Err(DecomposeInputDenial::InteriorKernelCoverageMismatch);
        }
        for identity in identities {
            if self.identities.as_slice().binary_search(identity).is_err() {
                return Err(DecomposeInputDenial::UnknownIdentity(*identity));
            }
        }
        Ok(())
    }

    fn merge_interiors(
        &self,
        identities: &[PartitionIdentity],
        values: Vec<InteriorResult<I, C>>,
    ) -> (Vec<I>, Vec<C>) {
        if let Some(previous) = &self.snapshot {
            let mut interiors = previous.interiors.clone();
            let mut contributions = previous.contributions.clone();
            for (identity, value) in identities.iter().zip(values) {
                let index = self
                    .identities
                    .as_slice()
                    .binary_search(identity)
                    .expect("checked identity");
                interiors[index] = value.interior;
                contributions[index] = value.contribution;
            }
            (interiors, contributions)
        } else {
            let (interiors, contributions) = values
                .into_iter()
                .map(|value| (value.interior, value.contribution))
                .unzip();
            (interiors, contributions)
        }
    }

    fn changed_contributions(
        &self,
        identities: &[PartitionIdentity],
        values: &[InteriorResult<I, C>],
    ) -> Result<Vec<PartitionIdentity>, DecomposeInputDenial> {
        let Some(previous) = &self.snapshot else {
            return Ok(identities.to_vec());
        };
        let mut changed = Vec::new();
        for (identity, value) in identities.iter().copied().zip(values) {
            let index = self
                .identities
                .as_slice()
                .binary_search(&identity)
                .expect("checked identity");
            if !same_encoding(
                &previous.contributions[index],
                &value.contribution,
                self.max_staged_bytes,
            )? {
                changed.push(identity);
            }
        }
        Ok(changed)
    }
}
