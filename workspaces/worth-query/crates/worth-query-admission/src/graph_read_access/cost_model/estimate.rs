use super::{
    WorthQueryGraphReadComplexityContract, WorthQueryGraphReadCostAttributionRow,
    WorthQueryGraphReadCostEstimateCounters, WorthQueryGraphReadCostEstimateStatus,
    WorthQueryGraphReadCostEvidence, WorthQueryGraphReadMemoryByteEstimate,
};
use std::fmt::{self, Write};
use worth_foundational::facade::CanonicalDigestId;

use crate::admission_digest::{hash_parts_with_digests, hash_parts_with_digests_admitted};
use crate::graph_read_access::digest_text::{admitted_digest_text, AdmittedDigestTextStop};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessCostEstimateDigest(String);

impl WorthQueryGraphReadAccessCostEstimateDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadIntrinsicCostEstimate {
    frontier_breadth: usize,
    edge_touches: usize,
    candidate_roots: usize,
    intermediate_set_size: usize,
}

impl WorthQueryGraphReadIntrinsicCostEstimate {
    pub fn frontier_breadth(&self) -> usize {
        self.frontier_breadth
    }

    pub fn edge_touches(&self) -> usize {
        self.edge_touches
    }

    pub fn candidate_roots(&self) -> usize {
        self.candidate_roots
    }

    pub fn intermediate_set_size(&self) -> usize {
        self.intermediate_set_size
    }

    pub(crate) fn new(
        frontier_breadth: usize,
        edge_touches: usize,
        candidate_roots: usize,
        intermediate_set_size: usize,
    ) -> Self {
        Self {
            frontier_breadth,
            edge_touches,
            candidate_roots,
            intermediate_set_size,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "intrinsic:frontier:{}:edges:{}:roots:{}:intermediate:{}",
            self.frontier_breadth,
            self.edge_touches,
            self.candidate_roots,
            self.intermediate_set_size
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadSupportedCostEstimate {
    memory: WorthQueryGraphReadMemoryByteEstimate,
    allocation_lifecycle_count: usize,
}

impl WorthQueryGraphReadSupportedCostEstimate {
    pub fn memory(&self) -> &WorthQueryGraphReadMemoryByteEstimate {
        &self.memory
    }

    pub fn index_bytes(&self) -> usize {
        self.memory.index_bytes()
    }

    pub fn result_bytes(&self) -> usize {
        self.memory.result_bytes()
    }

    pub fn proof_bytes(&self) -> usize {
        self.memory.proof_bytes()
    }

    pub fn allocation_lifecycle_count(&self) -> usize {
        self.allocation_lifecycle_count
    }

    pub(crate) fn new(
        memory: WorthQueryGraphReadMemoryByteEstimate,
        allocation_lifecycle_count: usize,
    ) -> Self {
        Self {
            memory,
            allocation_lifecycle_count,
        }
    }

    pub(crate) fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        output.write_str("supported:")?;
        self.memory.write_digest_part(output)?;
        write!(
            output,
            ":allocation_lifecycle:{}",
            self.allocation_lifecycle_count
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessCostEstimate {
    digest: WorthQueryGraphReadAccessCostEstimateDigest,
    requirement_set_digest: CanonicalDigestId,
    status: WorthQueryGraphReadCostEstimateStatus,
    complexity_contract: WorthQueryGraphReadComplexityContract,
    intrinsic: WorthQueryGraphReadIntrinsicCostEstimate,
    supported: WorthQueryGraphReadSupportedCostEstimate,
    counters: WorthQueryGraphReadCostEstimateCounters,
    attribution_rows: Vec<WorthQueryGraphReadCostAttributionRow>,
}

impl WorthQueryGraphReadAccessCostEstimate {
    pub fn digest(&self) -> &WorthQueryGraphReadAccessCostEstimateDigest {
        &self.digest
    }

    pub const fn requirement_set_digest(&self) -> &CanonicalDigestId {
        &self.requirement_set_digest
    }

    pub fn status(&self) -> &WorthQueryGraphReadCostEstimateStatus {
        &self.status
    }

    pub fn complexity_contract(&self) -> &WorthQueryGraphReadComplexityContract {
        &self.complexity_contract
    }

    pub fn intrinsic(&self) -> &WorthQueryGraphReadIntrinsicCostEstimate {
        &self.intrinsic
    }

    pub fn supported(&self) -> &WorthQueryGraphReadSupportedCostEstimate {
        &self.supported
    }

    pub fn counters(&self) -> &WorthQueryGraphReadCostEstimateCounters {
        &self.counters
    }

    pub fn attribution_rows(&self) -> &[WorthQueryGraphReadCostAttributionRow] {
        &self.attribution_rows
    }

    pub(crate) fn new(
        requirement_set_digest: CanonicalDigestId,
        evidence: &WorthQueryGraphReadCostEvidence,
        intrinsic: WorthQueryGraphReadIntrinsicCostEstimate,
        supported: WorthQueryGraphReadSupportedCostEstimate,
        counters: WorthQueryGraphReadCostEstimateCounters,
        attribution_rows: Vec<WorthQueryGraphReadCostAttributionRow>,
    ) -> Self {
        let complexity_contract = WorthQueryGraphReadComplexityContract::from_cost_dimensions(
            supported.index_bytes(),
            supported.result_bytes(),
            intrinsic.intermediate_set_size(),
        );
        let status = evidence.status().clone();
        let parts = vec![
            evidence.digest_part(),
            status.digest_part(),
            complexity_contract.digest_part(),
            intrinsic.digest_part(),
            supported.digest_part(),
            counters.digest_part(),
            attribution_rows_digest_part(&attribution_rows),
        ];
        Self::seal_parts(
            requirement_set_digest,
            status,
            complexity_contract,
            intrinsic,
            supported,
            counters,
            attribution_rows,
            parts,
        )
    }

    pub(crate) fn new_admitted<Stop>(
        requirement_set_digest: CanonicalDigestId,
        evidence: &WorthQueryGraphReadCostEvidence,
        intrinsic: WorthQueryGraphReadIntrinsicCostEstimate,
        supported: WorthQueryGraphReadSupportedCostEstimate,
        counters: WorthQueryGraphReadCostEstimateCounters,
        attribution_rows: Vec<WorthQueryGraphReadCostAttributionRow>,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, AdmittedDigestTextStop<Stop>> {
        let complexity_contract = WorthQueryGraphReadComplexityContract::from_cost_dimensions(
            supported.index_bytes(),
            supported.result_bytes(),
            intrinsic.intermediate_set_size(),
        );
        let status = evidence.status().clone();
        let slots = 7_u64
            .checked_mul(std::mem::size_of::<String>() as u64)
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        admit(7, slots).map_err(AdmittedDigestTextStop::Admission)?;
        let mut parts = Vec::with_capacity(7);
        parts.push(admitted_digest_text(
            4,
            |out| evidence.write_digest_part(out),
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            1,
            |out| status.write_digest_part(out),
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            1,
            |out| complexity_contract.write_digest_part(out),
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            4,
            |out| intrinsic.write_digest_part(out),
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            11,
            |out| supported.write_digest_part(out),
            &mut admit,
        )?);
        parts.push(admitted_digest_text(
            6,
            |out| counters.write_digest_part(out),
            &mut admit,
        )?);
        let row_visits = u64::try_from(attribution_rows.len())
            .ok()
            .and_then(|rows| rows.checked_mul(18))
            .and_then(|visits| visits.checked_add(1))
            .ok_or(AdmittedDigestTextStop::AccountingOverflow)?;
        parts.push(admitted_digest_text(
            row_visits,
            |out| write_attribution_rows_digest_part(&attribution_rows, out),
            &mut admit,
        )?);
        let digest =
            hash_parts_with_digests_admitted(&parts, &[&requirement_set_digest], &mut admit)?;
        Ok(Self::seal_digest(
            requirement_set_digest,
            status,
            complexity_contract,
            intrinsic,
            supported,
            counters,
            attribution_rows,
            digest,
        ))
    }

    fn seal_parts(
        requirement_set_digest: CanonicalDigestId,
        status: WorthQueryGraphReadCostEstimateStatus,
        complexity_contract: WorthQueryGraphReadComplexityContract,
        intrinsic: WorthQueryGraphReadIntrinsicCostEstimate,
        supported: WorthQueryGraphReadSupportedCostEstimate,
        counters: WorthQueryGraphReadCostEstimateCounters,
        attribution_rows: Vec<WorthQueryGraphReadCostAttributionRow>,
        parts: Vec<String>,
    ) -> Self {
        let digest = hash_parts_with_digests(&parts, &[&requirement_set_digest]);
        Self::seal_digest(
            requirement_set_digest,
            status,
            complexity_contract,
            intrinsic,
            supported,
            counters,
            attribution_rows,
            digest,
        )
    }

    fn seal_digest(
        requirement_set_digest: CanonicalDigestId,
        status: WorthQueryGraphReadCostEstimateStatus,
        complexity_contract: WorthQueryGraphReadComplexityContract,
        intrinsic: WorthQueryGraphReadIntrinsicCostEstimate,
        supported: WorthQueryGraphReadSupportedCostEstimate,
        counters: WorthQueryGraphReadCostEstimateCounters,
        attribution_rows: Vec<WorthQueryGraphReadCostAttributionRow>,
        digest: String,
    ) -> Self {
        Self {
            digest: WorthQueryGraphReadAccessCostEstimateDigest(digest),
            requirement_set_digest,
            status,
            complexity_contract,
            intrinsic,
            supported,
            counters,
            attribution_rows,
        }
    }
}

fn attribution_rows_digest_part(rows: &[WorthQueryGraphReadCostAttributionRow]) -> String {
    let mut text = String::new();
    write_attribution_rows_digest_part(rows, &mut text).expect("String formatting cannot fail");
    text
}

fn write_attribution_rows_digest_part(
    rows: &[WorthQueryGraphReadCostAttributionRow],
    output: &mut dyn Write,
) -> fmt::Result {
    output.write_str("attribution_rows:")?;
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            output.write_str("|")?;
        }
        row.write_digest_part(output)?;
    }
    Ok(())
}
