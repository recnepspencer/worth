use super::super::{
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalEquivalenceBasis, CanonicalizationCost,
    CanonicalizationRuleVersion,
};

pub struct CanonicalExportBundle {
    manifest: CanonicalExportManifest,
    bundle: CanonicalExportBasisBundle,
    harness_seed: CanonicalExportHarnessSeed,
    debt: Vec<CanonicalExportDebt>,
}

impl CanonicalExportBundle {
    pub(super) fn new(
        manifest: CanonicalExportManifest,
        bundle: CanonicalExportBasisBundle,
        harness_seed: CanonicalExportHarnessSeed,
        debt: Vec<CanonicalExportDebt>,
    ) -> Self {
        Self {
            manifest,
            bundle,
            harness_seed,
            debt,
        }
    }

    pub(in crate::canonicalization) fn into_basis(self) -> CanonicalExportBasisBundle {
        self.bundle
    }

    pub fn manifest(&self) -> &CanonicalExportManifest {
        &self.manifest
    }

    pub fn bundle(&self) -> &CanonicalExportBasisBundle {
        &self.bundle
    }

    pub fn harness_seed(&self) -> &CanonicalExportHarnessSeed {
        &self.harness_seed
    }

    pub fn debt(&self) -> &[CanonicalExportDebt] {
        &self.debt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExportBasisBundle {
    version: CanonicalizationRuleVersion,
    sequences: Vec<CanonicalExportBasisSequence>,
}

impl CanonicalExportBasisBundle {
    pub(super) fn new(
        version: CanonicalizationRuleVersion,
        sequences: Vec<CanonicalExportBasisSequence>,
    ) -> Self {
        Self { version, sequences }
    }

    pub(in crate::canonicalization) fn into_parts(
        self,
    ) -> (
        CanonicalizationRuleVersion,
        Vec<CanonicalExportBasisSequence>,
    ) {
        (self.version, self.sequences)
    }

    pub fn version(&self) -> &CanonicalizationRuleVersion {
        &self.version
    }

    pub fn sequences(&self) -> &[CanonicalExportBasisSequence] {
        &self.sequences
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExportBasisSequence {
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: Vec<CanonicalBasisEntry>,
    cost: CanonicalizationCost,
}

impl CanonicalExportBasisSequence {
    pub(super) fn from_ready_payload(
        version: CanonicalizationRuleVersion,
        domain: CanonicalBasisDomain,
        entries: &[CanonicalBasisEntry],
        cost: CanonicalizationCost,
    ) -> Self {
        Self {
            version,
            domain,
            entries: entries.to_vec(),
            cost,
        }
    }

    pub(in crate::canonicalization) fn into_parts(
        self,
    ) -> (
        CanonicalizationRuleVersion,
        CanonicalBasisDomain,
        Vec<CanonicalBasisEntry>,
        CanonicalizationCost,
    ) {
        (self.version, self.domain, self.entries, self.cost)
    }

    pub fn version(&self) -> &CanonicalizationRuleVersion {
        &self.version
    }

    pub const fn domain(&self) -> CanonicalBasisDomain {
        self.domain
    }

    pub fn entries(&self) -> &[CanonicalBasisEntry] {
        &self.entries
    }

    pub const fn cost(&self) -> CanonicalizationCost {
        self.cost
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExportManifest {
    fixture_name: String,
    rows: Vec<CanonicalExportManifestRow>,
}

impl CanonicalExportManifest {
    pub(super) fn new(
        fixture_name: impl Into<String>,
        rows: Vec<CanonicalExportManifestRow>,
    ) -> Self {
        Self {
            fixture_name: fixture_name.into(),
            rows,
        }
    }

    pub fn fixture_name(&self) -> &str {
        &self.fixture_name
    }

    pub fn rows(&self) -> &[CanonicalExportManifestRow] {
        &self.rows
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExportManifestRow {
    domain: CanonicalBasisDomain,
    rule_version: CanonicalizationRuleVersion,
    producer_shape: CanonicalProducerShape,
    equivalence_basis: CanonicalEquivalenceBasis,
    expected_entry_count: u32,
    expected_cost: CanonicalizationCost,
}

impl CanonicalExportManifestRow {
    pub(super) fn from_sequence(
        domain: CanonicalBasisDomain,
        rule_version: CanonicalizationRuleVersion,
        producer_shape: CanonicalProducerShape,
        equivalence_basis: CanonicalEquivalenceBasis,
        expected_entry_count: u32,
        expected_cost: CanonicalizationCost,
    ) -> Self {
        Self {
            domain,
            rule_version,
            producer_shape,
            equivalence_basis,
            expected_entry_count,
            expected_cost,
        }
    }

    pub const fn domain(&self) -> CanonicalBasisDomain {
        self.domain
    }

    pub fn rule_version(&self) -> &CanonicalizationRuleVersion {
        &self.rule_version
    }

    pub const fn producer_shape(&self) -> CanonicalProducerShape {
        self.producer_shape
    }

    pub const fn equivalence_basis(&self) -> CanonicalEquivalenceBasis {
        self.equivalence_basis
    }

    pub const fn expected_entry_count(&self) -> u32 {
        self.expected_entry_count
    }

    pub const fn expected_cost(&self) -> CanonicalizationCost {
        self.expected_cost
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalProducerShape {
    NativeFoundational,
    CompatibilityLowered,
    GoldenFixture,
    SupportReplay,
    WORTHHarnessSeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalExportDebt {
    FinalDigestPolicyDeferred,
    RuntimeAdoptionParityDeferred,
    LaterMilestoneDomainDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalExportHarnessSeed {
    lane: &'static str,
    replay_scope: &'static str,
}

impl CanonicalExportHarnessSeed {
    pub const fn new(lane: &'static str, replay_scope: &'static str) -> Self {
        Self { lane, replay_scope }
    }

    pub const fn lane(&self) -> &'static str {
        self.lane
    }

    pub const fn replay_scope(&self) -> &'static str {
        self.replay_scope
    }
}
