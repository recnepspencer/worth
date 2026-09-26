use worth_foundational::{
    performance_api::{
        common_path,
        lower_lane::{basis, receipts},
    },
    FoundationalAuthoritativePerformanceClaim, FoundationalCounterBackedPerformanceReceipt,
    FoundationalPerformanceContractName, FoundationalPerformanceCounterName,
    FoundationalPerformanceCounterRow, FoundationalPerformanceCounterSpec,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceWorkClass,
};
use worth_proof::TransitionOutcome;

use super::certification::WorthUiCertifiedMeasurementPacket;
use super::denial::WorthUiMeasurementCertificationDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorthUiFoundationalCounterEvidence {
    counter_specs: Vec<FoundationalPerformanceCounterSpec>,
    counter_rows: Vec<FoundationalPerformanceCounterRow>,
    counter_backed_receipt:
        FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
    canonical_basis_entry_count: u32,
    worth_ui_replay_digest: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WorthUiFoundationalCounterBridge;

impl WorthUiFoundationalCounterBridge {
    pub fn lower_certified_packet(
        certified: &WorthUiCertifiedMeasurementPacket,
    ) -> Result<WorthUiFoundationalCounterEvidence, WorthUiMeasurementCertificationDenial> {
        let mut counter_specs = Vec::new();
        let mut counter_rows = Vec::new();
        for counter in certified.packet().counters() {
            if !counter.certifies_execution_work() {
                continue;
            }
            let name = FoundationalPerformanceCounterName::new(counter.name()).map_err(|_| {
                WorthUiMeasurementCertificationDenial::InvalidFoundationalCounterName
            })?;
            counter_specs.push(FoundationalPerformanceCounterSpec::new(
                name.clone(),
                counter.work_class(),
                counter.value(),
            ));
            counter_rows.push(FoundationalPerformanceCounterRow::new(
                name,
                counter.value(),
            ));
        }
        if counter_rows.is_empty() {
            return Err(WorthUiMeasurementCertificationDenial::MissingWorthUiCounterEvidence);
        }
        let counter_backed_receipt =
            build_foundational_counter_backed_receipt(certified, &counter_specs, &counter_rows)?;
        let canonical_basis_entry_count =
            match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
                basis::performance_basis_rule_version(),
                &counter_backed_receipt,
            ) {
                TransitionOutcome::Success(ready) => ready.payload().cost().entry_count(),
                TransitionOutcome::Denied(_)
                | TransitionOutcome::Deferred(_)
                | TransitionOutcome::Stale(_)
                | TransitionOutcome::RebindRequired(_)
                | TransitionOutcome::Failed(_) => {
                    return Err(
                        WorthUiMeasurementCertificationDenial::FoundationalBasisConstruction,
                    )
                }
            };
        Ok(WorthUiFoundationalCounterEvidence {
            counter_specs,
            counter_rows,
            counter_backed_receipt,
            canonical_basis_entry_count,
            worth_ui_replay_digest: certified.replay_digest(),
        })
    }
}

impl WorthUiFoundationalCounterEvidence {
    pub fn counter_specs(&self) -> &[FoundationalPerformanceCounterSpec] {
        &self.counter_specs
    }

    pub fn counter_rows(&self) -> &[FoundationalPerformanceCounterRow] {
        &self.counter_rows
    }

    pub fn counter_backed_receipt(
        &self,
    ) -> &FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>
    {
        &self.counter_backed_receipt
    }

    pub fn canonical_basis_entry_count(&self) -> u32 {
        self.canonical_basis_entry_count
    }

    pub fn worth_ui_replay_digest(&self) -> u64 {
        self.worth_ui_replay_digest
    }
}

fn build_foundational_counter_backed_receipt(
    certified: &WorthUiCertifiedMeasurementPacket,
    counter_specs: &[FoundationalPerformanceCounterSpec],
    counter_rows: &[FoundationalPerformanceCounterRow],
) -> Result<
    FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
    WorthUiMeasurementCertificationDenial,
> {
    let common = common_path::performance();
    let mut claim_builder = common
        .claim()
        .authoritative_execution()
        .boundary(certified.contract().foundational_boundary_value())
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(certified.contract().breadth_locality_value())
        .access_pattern(certified.contract().access_pattern_value())
        .execution_temperature(certified.contract().execution_temperature_value())
        .freshness_retention(certified.contract().freshness_retention_value())
        .fallback_debt(certified.contract().fallback_debt_value());

    for work_class in included_work_classes(counter_specs) {
        claim_builder = claim_builder.include_work(work_class);
    }
    for work_class in excluded_work_classes(counter_specs) {
        claim_builder = claim_builder.exclude_work(work_class);
    }

    let claim = claim_builder
        .finish()
        .map_err(|_| WorthUiMeasurementCertificationDenial::FoundationalClaimConstruction)?;
    let mut bundle = basis::performance_bundle(claim).attach_contract_name(
        FoundationalPerformanceContractName::new(certified.contract().name())
            .map_err(|_| WorthUiMeasurementCertificationDenial::InvalidFoundationalCounterName)?,
    );
    for spec in counter_specs {
        bundle = bundle.attach_counter_spec(spec.clone());
    }
    let bundle = bundle
        .finish()
        .map_err(|_| WorthUiMeasurementCertificationDenial::FoundationalBundleConstruction)?;
    let mut receipt = receipts::counter_backed_performance_receipt(bundle);
    for row in counter_rows {
        receipt = receipt.attach_counter_row(row.clone());
    }
    receipt
        .finish()
        .map_err(|_| WorthUiMeasurementCertificationDenial::FoundationalReceiptConstruction)
}

fn included_work_classes(
    counter_specs: &[FoundationalPerformanceCounterSpec],
) -> Vec<FoundationalPerformanceWorkClass> {
    let mut work_classes: Vec<_> = counter_specs.iter().map(|spec| spec.work_class()).collect();
    work_classes.sort();
    work_classes.dedup();
    work_classes
}

fn excluded_work_classes(
    counter_specs: &[FoundationalPerformanceCounterSpec],
) -> Vec<FoundationalPerformanceWorkClass> {
    [
        FoundationalPerformanceWorkClass::ReplayReconstruction,
        FoundationalPerformanceWorkClass::SupportReportAssembly,
        FoundationalPerformanceWorkClass::ForensicParity,
        FoundationalPerformanceWorkClass::PublicationDelivery,
    ]
    .into_iter()
    .filter(|work_class| {
        counter_specs
            .iter()
            .all(|spec| spec.work_class() != *work_class)
    })
    .collect()
}
