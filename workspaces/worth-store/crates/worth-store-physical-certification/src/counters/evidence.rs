use std::collections::{BTreeMap, BTreeSet};

use super::{
    build_foundational_receipt, CounterContractKind, CounterExpectationKind,
    CounterMismatchEvidence, PhysicalCounterContract, PhysicalExecutedCounterEvidence,
    PhysicalResidencyEvidenceSource,
};
use crate::{PhysicalSimulationPlan, PhysicalSimulationPlanIdentity, PhysicalSimulationProfile};
use worth_foundational::{
    FoundationalAuthoritativePerformanceClaim, FoundationalCounterBackedPerformanceReceipt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalCounterEvidenceRow {
    kind: CounterContractKind,
    strength: CounterExpectationKind,
    observed_count: u64,
    previous_observed_count: Option<u64>,
}

impl PhysicalCounterEvidenceRow {
    pub(crate) const fn new(
        kind: CounterContractKind,
        strength: CounterExpectationKind,
        observed_count: u64,
    ) -> Self {
        Self {
            kind,
            strength,
            observed_count,
            previous_observed_count: None,
        }
    }

    pub const fn with_previous_observed_count(mut self, previous_observed_count: u64) -> Self {
        self.previous_observed_count = Some(previous_observed_count);
        self
    }

    pub(crate) const fn from_hostile_readmission(
        kind: CounterContractKind,
        strength: CounterExpectationKind,
        observed_count: u64,
        previous_observed_count: Option<u64>,
    ) -> Self {
        Self {
            kind,
            strength,
            observed_count,
            previous_observed_count,
        }
    }

    pub const fn kind(self) -> CounterContractKind {
        self.kind
    }

    pub const fn strength(self) -> CounterExpectationKind {
        self.strength
    }

    pub const fn observed_count(self) -> u64 {
        self.observed_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalResourceEnvelopeObservation {
    profile: PhysicalSimulationProfile,
    allocation_bytes: u64,
    resident_bytes: u64,
    pinned_pages: u64,
    dirty_pages: u64,
    io_queue_depth: u64,
    io_interference_events: u64,
}

impl PhysicalResourceEnvelopeObservation {
    pub(crate) const fn new(
        profile: PhysicalSimulationProfile,
        allocation_bytes: u64,
        resident_bytes: u64,
        pinned_pages: u64,
        dirty_pages: u64,
        io_queue_depth: u64,
        io_interference_events: u64,
    ) -> Self {
        Self {
            profile,
            allocation_bytes,
            resident_bytes,
            pinned_pages,
            dirty_pages,
            io_queue_depth,
            io_interference_events,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalCounterEvidenceReceipt {
    plan_identity: PhysicalSimulationPlanIdentity,
    rows: Vec<PhysicalCounterEvidenceRow>,
    residency_source: PhysicalResidencyEvidenceSource,
    foundational:
        FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
}

impl PhysicalCounterEvidenceReceipt {
    pub const fn plan_identity(&self) -> &PhysicalSimulationPlanIdentity {
        &self.plan_identity
    }

    pub fn rows(&self) -> &[PhysicalCounterEvidenceRow] {
        &self.rows
    }

    pub const fn residency_source(&self) -> PhysicalResidencyEvidenceSource {
        self.residency_source
    }

    pub const fn foundational_receipt(
        &self,
    ) -> &FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>
    {
        &self.foundational
    }
}

pub fn admit_physical_counter_evidence(
    plan: &PhysicalSimulationPlan,
    evidence: PhysicalExecutedCounterEvidence,
) -> Result<PhysicalCounterEvidenceReceipt, CounterMismatchEvidence> {
    require_resource_observation_within_envelope(plan, evidence.resource_observation)?;
    let rows = sorted_counter_rows(evidence.rows)?;
    require_expected_counter_rows(plan, &rows)?;
    let foundational = build_foundational_receipt(&rows)?;
    Ok(PhysicalCounterEvidenceReceipt {
        plan_identity: plan.identity().clone(),
        rows,
        residency_source: evidence.residency_source,
        foundational,
    })
}

pub(crate) fn sorted_counter_rows(
    rows: impl IntoIterator<Item = PhysicalCounterEvidenceRow>,
) -> Result<Vec<PhysicalCounterEvidenceRow>, CounterMismatchEvidence> {
    let mut seen = BTreeSet::new();
    let mut sorted = rows.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|row| row.kind);
    for row in &sorted {
        if !seen.insert(row.kind) {
            return Err(CounterMismatchEvidence::DuplicateCounterRow { kind: row.kind });
        }
    }
    Ok(sorted)
}

pub(crate) fn require_expected_counter_rows(
    plan: &PhysicalSimulationPlan,
    rows: &[PhysicalCounterEvidenceRow],
) -> Result<(), CounterMismatchEvidence> {
    let rows_by_kind = rows
        .iter()
        .map(|row| (row.kind, row))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        if !plan.counter_contracts().contains(row.kind) {
            return Err(CounterMismatchEvidence::UnexpectedCounterRow { kind: row.kind });
        }
    }
    for contract in plan.counter_contracts().iter() {
        let Some(row) = rows_by_kind.get(&contract.kind()) else {
            return Err(CounterMismatchEvidence::MissingCounterSpec {
                kind: contract.kind(),
            });
        };
        require_counter_row_satisfies_contract(contract, row)?;
    }
    Ok(())
}

fn require_counter_row_satisfies_contract(
    contract: &PhysicalCounterContract,
    row: &PhysicalCounterEvidenceRow,
) -> Result<(), CounterMismatchEvidence> {
    if row.strength != contract.expectation().kind() {
        return Err(CounterMismatchEvidence::UnderStrengthEvidence {
            kind: contract.kind(),
            required: contract.expectation().kind(),
            actual: row.strength,
        });
    }
    match contract.expectation().kind() {
        CounterExpectationKind::Zero => require_zero_counter(contract, row),
        CounterExpectationKind::Positive => require_positive_counter(contract, row),
        CounterExpectationKind::Exact => require_exact_counter(contract, row),
        CounterExpectationKind::Monotonic => require_monotonic_counter(contract, row),
        CounterExpectationKind::Bounded => require_bounded_counter(contract, row),
        CounterExpectationKind::ProfileScoped => Ok(()),
    }
}

fn require_zero_counter(
    contract: &PhysicalCounterContract,
    row: &PhysicalCounterEvidenceRow,
) -> Result<(), CounterMismatchEvidence> {
    if row.observed_count != 0 {
        return Err(CounterMismatchEvidence::NonZeroForbiddenCounter {
            kind: contract.kind(),
            actual: row.observed_count,
        });
    }
    Ok(())
}

fn require_positive_counter(
    contract: &PhysicalCounterContract,
    row: &PhysicalCounterEvidenceRow,
) -> Result<(), CounterMismatchEvidence> {
    if row.observed_count == 0 {
        return Err(CounterMismatchEvidence::PositiveCounterNotPositive {
            kind: contract.kind(),
            actual: row.observed_count,
        });
    }
    Ok(())
}

fn require_exact_counter(
    contract: &PhysicalCounterContract,
    row: &PhysicalCounterEvidenceRow,
) -> Result<(), CounterMismatchEvidence> {
    let expected = contract
        .expectation()
        .value()
        .expect("exact contracts bind values");
    if row.observed_count != expected {
        if contract.kind() == CounterContractKind::ForbiddenShortcutExact && expected == 0 {
            return Err(CounterMismatchEvidence::NonZeroForbiddenCounter {
                kind: contract.kind(),
                actual: row.observed_count,
            });
        }
        return Err(CounterMismatchEvidence::CounterValueMismatch {
            kind: contract.kind(),
            expected,
            actual: row.observed_count,
        });
    }
    Ok(())
}

fn require_monotonic_counter(
    contract: &PhysicalCounterContract,
    row: &PhysicalCounterEvidenceRow,
) -> Result<(), CounterMismatchEvidence> {
    if let Some(previous) = row.previous_observed_count
        && row.observed_count < previous
    {
        return Err(CounterMismatchEvidence::MonotonicCounterRegressed {
            kind: contract.kind(),
            previous,
            actual: row.observed_count,
        });
    }
    Ok(())
}

fn require_bounded_counter(
    contract: &PhysicalCounterContract,
    row: &PhysicalCounterEvidenceRow,
) -> Result<(), CounterMismatchEvidence> {
    let maximum = contract
        .expectation()
        .value()
        .expect("bounded contracts bind values");
    if row.observed_count > maximum {
        return Err(CounterMismatchEvidence::BoundedCounterExceeded {
            kind: contract.kind(),
            maximum,
            actual: row.observed_count,
        });
    }
    Ok(())
}

pub(crate) fn require_resource_observation_within_envelope(
    plan: &PhysicalSimulationPlan,
    observation: PhysicalResourceEnvelopeObservation,
) -> Result<(), CounterMismatchEvidence> {
    let envelope = plan.resource_envelope();
    if observation.profile != envelope.profile() {
        return Err(CounterMismatchEvidence::ProfileMismatch {
            expected: envelope.profile(),
            actual: observation.profile,
        });
    }
    require_resource_bound(
        CounterContractKind::AllocationBytes,
        envelope.allocation_bytes(),
        observation.allocation_bytes,
    )?;
    require_resource_bound(
        CounterContractKind::ResidentBytes,
        envelope.resident_bytes(),
        observation.resident_bytes,
    )?;
    require_resource_bound(
        CounterContractKind::PagePins,
        u64::from(envelope.max_pinned_pages()),
        observation.pinned_pages,
    )?;
    require_resource_bound(
        CounterContractKind::DirtyPages,
        u64::from(envelope.max_dirty_pages()),
        observation.dirty_pages,
    )?;
    require_resource_bound(
        CounterContractKind::IoQueueDepth,
        u64::from(envelope.io_queue().max_queue_depth()),
        observation.io_queue_depth,
    )?;
    require_resource_bound(
        CounterContractKind::IoInterferenceEvents,
        u64::from(envelope.io_queue().max_interference_events()),
        observation.io_interference_events,
    )
}

fn require_resource_bound(
    kind: CounterContractKind,
    maximum: u64,
    actual: u64,
) -> Result<(), CounterMismatchEvidence> {
    if actual > maximum {
        return Err(CounterMismatchEvidence::ResourceEnvelopeExceeded {
            kind,
            maximum,
            actual,
        });
    }
    Ok(())
}
