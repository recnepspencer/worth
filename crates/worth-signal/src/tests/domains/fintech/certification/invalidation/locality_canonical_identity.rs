use crate::data::error::SignalError;
use crate::data::proof::{
    attach_foundational_invalidation_performance_receipt, SignalInvalidationExecutionReceipt,
};
use crate::tests::domains::fintech::world::{
    DensityRatio, FinancialLocalityDefinition, FinancialLocalityScenario,
    FinancialLocalityTraceIdentity, LocalityLane, LocalityScaleTuple, RestorePosture,
    SparseFanoutAxis,
};
use worth_foundational::facade::{
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalIntegerWidth,
};

mod basis;

use super::{FinancialCanonicalCaseIdentity, FinancialLocalityExpectationManifest};

const CASE_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("WORTH.signal.financial-certification-case");

pub(in crate::tests::domains::fintech) fn verified_locality_case_identity(
    definition: &FinancialLocalityDefinition,
    manifest: &FinancialLocalityExpectationManifest,
    diagnostics_tier: crate::facade::DiagnosticsTier,
    performed: SignalInvalidationExecutionReceipt,
    physical_batches: u64,
    physical_peak: u64,
) -> Result<FinancialCanonicalCaseIdentity, SignalError> {
    if manifest.scenario() != definition.scenario() {
        return Err(SignalError::invalid_input(
            "locality receipt manifest belongs to another scenario",
        ));
    }
    let expected =
        super::locality_receipt::expected_counters(manifest, physical_batches, physical_peak);
    attach_foundational_invalidation_performance_receipt(performed, expected).map_err(
        |denial| {
            SignalError::invalid_input(format!(
                "locality performed receipt did not match its manifest: {denial:?}"
            ))
        },
    )?;
    locality_case_identity(definition, manifest, diagnostics_tier)
}

fn locality_case_identity(
    definition: &FinancialLocalityDefinition,
    manifest: &FinancialLocalityExpectationManifest,
    diagnostics_tier: crate::facade::DiagnosticsTier,
) -> Result<FinancialCanonicalCaseIdentity, SignalError> {
    let trace = manifest.action_trace();
    let mut entries = vec![
        text_entry(
            CASE_DOMAIN,
            "scenario",
            scenario_name(definition.scenario()),
        ),
        unsigned_entry(CASE_DOMAIN, "seed", definition.seed() as u128),
        text_entry(CASE_DOMAIN, "trace", trace_name(trace)),
        unsigned_entry(CASE_DOMAIN, "trace.ordinal", trace_ordinal(trace)),
        text_entry(
            CASE_DOMAIN,
            "lane",
            lane_name(definition.workload().execution_posture()),
        ),
    ];
    entries.extend(scale_entries(definition.scale()));
    entries.extend(basis::identity_entries(
        definition,
        manifest,
        diagnostics_tier,
    )?);
    FinancialCanonicalCaseIdentity::from_extended_entries(entries)
}

fn scale_entries(scale: LocalityScaleTuple) -> Vec<CanonicalBasisEntry> {
    let (kind, first, second, third) = match scale {
        LocalityScaleTuple::SparseBookFanout {
            total_outputs,
            axis,
        } => (sparse_axis_name(axis), u128::from(total_outputs), 0, 0),
        LocalityScaleTuple::PartitionedCurveUniverse {
            regions,
            matching_memberships,
            instruments_per_matching_region,
        } => (
            "partitioned",
            u128::from(regions),
            u128::from(matching_memberships),
            u128::from(instruments_per_matching_region),
        ),
        LocalityScaleTuple::ConvergentFactorBatch {
            producer_permutations,
            duplicate_admissions,
            canonical_seeds,
        } => (
            "convergent",
            u128::from(producer_permutations),
            u128::from(duplicate_admissions),
            u128::from(canonical_seeds),
        ),
        LocalityScaleTuple::DenseMarketClose {
            total_outputs,
            affected_ratio,
        } => (
            density_name(affected_ratio),
            u128::from(total_outputs),
            0,
            0,
        ),
        LocalityScaleTuple::PortfolioDependencyChurn {
            rounds,
            canonical_seeds,
        } => ("churn", u128::from(rounds), u128::from(canonical_seeds), 0),
        LocalityScaleTuple::BranchRestoreLocalityReplay {
            posture,
            total_outputs,
            canonical_seeds,
        } => (
            restore_name(posture),
            u128::from(total_outputs),
            u128::from(canonical_seeds),
            0,
        ),
    };
    vec![
        text_entry(CASE_DOMAIN, "scale.kind", kind),
        unsigned_entry(CASE_DOMAIN, "scale.first", first),
        unsigned_entry(CASE_DOMAIN, "scale.second", second),
        unsigned_entry(CASE_DOMAIN, "scale.third", third),
    ]
}

fn text_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: &'static str,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

fn unsigned_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: u128,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits128,
            value,
        },
    )
}

const fn scenario_name(value: FinancialLocalityScenario) -> &'static str {
    match value {
        FinancialLocalityScenario::SparseBookFanout => "sparse-book-fanout",
        FinancialLocalityScenario::PartitionedCurveUniverse => "partitioned-curve-universe",
        FinancialLocalityScenario::ConvergentFactorBatch => "convergent-factor-batch",
        FinancialLocalityScenario::DenseMarketClose => "dense-market-close",
        FinancialLocalityScenario::PortfolioDependencyChurn => "portfolio-dependency-churn",
        FinancialLocalityScenario::BranchRestoreLocalityReplay => "branch-restore-replay",
    }
}

const fn trace_name(value: FinancialLocalityTraceIdentity) -> &'static str {
    match value {
        FinancialLocalityTraceIdentity::PrimaryMutation => "primary",
        FinancialLocalityTraceIdentity::PartitionWholeRegion => "partition-whole",
        FinancialLocalityTraceIdentity::PartitionCorrelatedScopes => "partition-correlated",
        FinancialLocalityTraceIdentity::ProducerPermutation(_) => "producer-permutation",
        FinancialLocalityTraceIdentity::PortfolioChurn => "portfolio-churn",
        FinancialLocalityTraceIdentity::BranchRestoreReplay => "branch-restore-replay",
    }
}

const fn trace_ordinal(value: FinancialLocalityTraceIdentity) -> u128 {
    match value {
        FinancialLocalityTraceIdentity::ProducerPermutation(ordinal) => ordinal as u128,
        _ => 0,
    }
}

const fn lane_name(value: LocalityLane) -> &'static str {
    match value {
        LocalityLane::OrdinaryChangeGate => "ordinary",
        LocalityLane::Scheduled => "scheduled",
    }
}

const fn sparse_axis_name(value: SparseFanoutAxis) -> &'static str {
    match value {
        SparseFanoutAxis::IndexDisjoint => "sparse-index-disjoint",
        SparseFanoutAxis::QueriedRejecting => "sparse-queried-rejecting",
        SparseFanoutAxis::RejectedDescendants => "sparse-rejected-descendants",
    }
}

const fn density_name(value: DensityRatio) -> &'static str {
    match value {
        DensityRatio::OneInOneHundred => "dense-one-in-one-hundred",
        DensityRatio::OneInFour => "dense-one-in-four",
        DensityRatio::FourInFive => "dense-four-in-five",
    }
}

const fn restore_name(value: RestorePosture) -> &'static str {
    match value {
        RestorePosture::Narrow => "restore-narrow",
        RestorePosture::Convergent => "restore-convergent",
        RestorePosture::DenseFourInFive => "restore-dense-four-in-five",
    }
}
