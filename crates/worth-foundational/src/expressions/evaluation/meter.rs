//! The semantic evaluation meter and its versioned cost contract.
//!
//! One work unit is charged per evaluated program node, per loop iteration,
//! per started 8-byte word inspected, compared, or copied, and per 64-bit
//! limb a bus operation processes. Charges are checked before the work they
//! pay for, so a denial happens before a limit would be exceeded. A slice
//! quantum bounds the work between suspension points: a step that does not
//! fit the rest of a slice waits for the next one, and a fresh slice always
//! admits its first step, which never exceeds [`MAX_SLICE_QUANTUM`]. Charges
//! never depend on where a slice ends.

use std::task::Poll;

use crate::expressions::denial::{ExpressionDenial, ExpressionResource, ExpressionResult};
use crate::expressions::profile::ExpressionProfile;

/// The largest slice quantum, in semantic work units.
pub const MAX_SLICE_QUANTUM: u64 = 4_096;

/// The cost contract version bound into evaluation results.
pub(crate) const COST_CONTRACT: &str = "worth-expression-cost-v1";

const LOGICAL: [ExpressionResource; 6] = [
    ExpressionResource::SemanticWork,
    ExpressionResource::VisitedElements,
    ExpressionResource::InputBytes,
    ExpressionResource::OutputBytes,
    ExpressionResource::ScratchBytes,
    ExpressionResource::RetainedConsumptionBytes,
];

/// Logical and physical cost of one evaluation, cumulative across slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ExpressionCost {
    logical: [u64; 6],
    reads: u64,
    calls: u64,
    comparisons: u64,
    allocations: u64,
    copied_bytes: u64,
    slices: u64,
}

impl ExpressionCost {
    /// Logical use of `resource`; zero for resources evaluation does not meter.
    pub fn used(&self, resource: ExpressionResource) -> u64 {
        LOGICAL
            .iter()
            .position(|metered| *metered == resource)
            .map_or(0, |index| self.logical[index])
    }

    /// Operand, field, local, and element reads.
    pub fn reads(&self) -> u64 {
        self.reads
    }

    /// Installed function calls executed.
    pub fn calls(&self) -> u64 {
        self.calls
    }

    /// Value comparisons started.
    pub fn comparisons(&self) -> u64 {
        self.comparisons
    }

    /// Values allocated. Allocated bytes are charged as scratch, which bounds
    /// peak scratch from above.
    pub fn allocations(&self) -> u64 {
        self.allocations
    }

    /// String and Bytes bytes copied into new values.
    pub fn copied_bytes(&self) -> u64 {
        self.copied_bytes
    }

    /// Slices the evaluation ran in, including the final one.
    pub fn slices(&self) -> u64 {
        self.slices
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EvaluationMeter {
    limits: [u64; 6],
    cost: ExpressionCost,
    /// Work units left in the current slice.
    slice: u64,
    /// Whether the current slice has admitted no step yet.
    fresh: bool,
}

impl EvaluationMeter {
    pub(crate) fn new(profile: &ExpressionProfile) -> Self {
        Self {
            limits: LOGICAL.map(|resource| profile.limit(resource)),
            cost: ExpressionCost::default(),
            slice: 0,
            fresh: false,
        }
    }

    /// Opens a slice of at most `quantum` units, clamped to `1..=MAX`.
    pub(crate) fn open_slice(&mut self, quantum: u64) {
        self.slice = quantum.clamp(1, MAX_SLICE_QUANTUM);
        self.fresh = true;
        self.cost.slices += 1;
    }

    pub(crate) fn cost(&self) -> ExpressionCost {
        self.cost
    }

    fn charge(&mut self, index: usize, units: u64) -> ExpressionResult<()> {
        let used = self.cost.logical[index].checked_add(units);
        match used {
            Some(used) if used <= self.limits[index] => {
                self.cost.logical[index] = used;
                Ok(())
            }
            _ => Err(ExpressionDenial::resource(
                LOGICAL[index],
                self.limits[index],
            )),
        }
    }

    /// Charges `units` of atomic work, or asks to yield when the rest of the
    /// slice cannot pay for it.
    pub(crate) fn work(&mut self, units: u64) -> ExpressionResult<Poll<()>> {
        debug_assert!(units <= MAX_SLICE_QUANTUM, "atomic steps fit a slice");
        if units > self.slice && !self.fresh {
            return Ok(Poll::Pending);
        }
        self.charge(0, units)?;
        self.slice = self.slice.saturating_sub(units);
        self.fresh = false;
        Ok(Poll::Ready(()))
    }

    /// Pays `owed` units across as many slices as it takes, for a pass whose
    /// cost is known before it starts. The whole charge is checked against the
    /// limit first, so a denial never depends on where slices end.
    pub(crate) fn prepay(&mut self, owed: &mut u64) -> ExpressionResult<Poll<()>> {
        if self.cost.logical[0].saturating_add(*owed) > self.limits[0] {
            return Err(ExpressionDenial::resource(LOGICAL[0], self.limits[0]));
        }
        while *owed > 0 {
            let units = (*owed).min(self.slice.max(1)).min(MAX_SLICE_QUANTUM);
            if self.work(units)?.is_pending() {
                return Ok(Poll::Pending);
            }
            *owed -= units;
        }
        Ok(Poll::Ready(()))
    }

    pub(crate) fn visit(&mut self, elements: u64) -> ExpressionResult<()> {
        self.charge(1, elements)
    }

    pub(crate) fn input(&mut self, bytes: u64) -> ExpressionResult<()> {
        self.charge(2, bytes)
    }

    pub(crate) fn output(&mut self, bytes: u64) -> ExpressionResult<()> {
        self.charge(3, bytes)
    }

    /// Charges an allocation of `bytes` before it is made.
    pub(crate) fn allocate(&mut self, bytes: u64) -> ExpressionResult<()> {
        self.charge(4, bytes)?;
        self.cost.allocations += 1;
        Ok(())
    }

    /// Charges scratch that grows an allocation already counted.
    pub(crate) fn scratch(&mut self, bytes: u64) -> ExpressionResult<()> {
        self.charge(4, bytes)
    }

    pub(crate) fn retain(&mut self, bytes: u64) -> ExpressionResult<()> {
        self.charge(5, bytes)
    }

    pub(crate) fn read(&mut self) {
        self.cost.reads += 1;
    }

    pub(crate) fn copied(&mut self, bytes: u64) {
        self.cost.copied_bytes += bytes;
    }

    pub(crate) fn call(&mut self) {
        self.cost.calls += 1;
    }

    pub(crate) fn compare(&mut self) {
        self.cost.comparisons += 1;
    }
}
