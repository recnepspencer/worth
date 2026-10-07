use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use crate::canonical_identity_derivation::WorthQueryCanonicalIdentityBasis;

use super::{WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow};

mod admitted;
mod canonical_basis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessRequirementSetDigest(CanonicalDigestId);

impl WorthQueryGraphReadAccessRequirementSetDigest {
    pub const fn as_digest(&self) -> &CanonicalDigestId {
        &self.0
    }

    pub fn render_support_hex(&self) -> String {
        self.0.render_hex()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessRequirementSet {
    digest: WorthQueryGraphReadAccessRequirementSetDigest,
    read_graph_digest: CanonicalDigestId,
    access_shape_digest: CanonicalDigestId,
    selectivity_shape_digest: CanonicalDigestId,
    rows: Vec<WorthQueryGraphReadAccessRequirementRow>,
    counters: WorthQueryGraphReadAccessRequirementCounters,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryGraphReadAccessRequirementSet {
    pub fn new(
        read_graph_digest: CanonicalDigestId,
        access_shape_digest: CanonicalDigestId,
        selectivity_shape_digest: CanonicalDigestId,
        rows: Vec<WorthQueryGraphReadAccessRequirementRow>,
        budget: CanonicalDigestWorkBudget,
        prior_work: WorthQueryCanonicalWorkEvidence,
    ) -> Result<Self, CanonicalDigestDerivationDenial> {
        // Render each semantic key once; comparisons borrow it. The stable
        // order and row equality remain the existing canonical contract.
        let mut keyed: Vec<_> = rows
            .into_iter()
            .map(|row| (row.digest_part(), row))
            .collect();
        keyed.sort_by(|left, right| left.0.cmp(&right.0));
        let mut rows: Vec<_> = keyed.into_iter().map(|(_, row)| row).collect();
        rows.dedup();
        let counters = WorthQueryGraphReadAccessRequirementCounters::from_rows(&rows);
        let (digest, work) = derive_requirement_set_digest(
            read_graph_digest,
            access_shape_digest,
            selectivity_shape_digest,
            &rows,
            budget,
        )?;
        Ok(Self {
            digest: WorthQueryGraphReadAccessRequirementSetDigest(digest),
            read_graph_digest,
            access_shape_digest,
            selectivity_shape_digest,
            rows,
            counters,
            canonical_work: prior_work.combine(work),
        })
    }

    pub fn digest(&self) -> &WorthQueryGraphReadAccessRequirementSetDigest {
        &self.digest
    }

    pub const fn read_graph_digest(&self) -> &CanonicalDigestId {
        &self.read_graph_digest
    }

    pub const fn access_shape_digest(&self) -> &CanonicalDigestId {
        &self.access_shape_digest
    }

    pub const fn selectivity_shape_digest(&self) -> &CanonicalDigestId {
        &self.selectivity_shape_digest
    }

    pub fn rows(&self) -> &[WorthQueryGraphReadAccessRequirementRow] {
        &self.rows
    }

    pub fn counters(&self) -> &WorthQueryGraphReadAccessRequirementCounters {
        &self.counters
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }

    pub fn diagnostic_canonical_parts(&self) -> Vec<String> {
        let mut parts = vec![
            format!("read_graph:{}", self.read_graph_digest.render_hex()),
            format!("access_shape:{}", self.access_shape_digest.render_hex()),
            format!(
                "selectivity_shape:{}",
                self.selectivity_shape_digest.render_hex()
            ),
            format!("row_count:{}", self.rows.len()),
        ];
        parts.extend(self.rows.iter().map(|row| row.digest_part()));
        parts
    }

    pub fn contains_kind(&self, kind: &WorthQueryGraphReadAccessRequirementKind) -> bool {
        self.rows.iter().any(|row| row.kind() == kind)
    }

    pub fn requires_kind(&self, kind: WorthQueryGraphReadAccessRequirementKind) -> bool {
        self.contains_kind(&kind)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessRequirementCounters {
    row_count: usize,
    counts: [usize; 12],
}

impl WorthQueryGraphReadAccessRequirementCounters {
    fn from_rows(rows: &[WorthQueryGraphReadAccessRequirementRow]) -> Self {
        let kinds = WorthQueryGraphReadAccessRequirementKind::all();
        let mut counts = [0; 12];
        for (index, kind) in kinds.iter().enumerate() {
            counts[index] = rows.iter().filter(|row| row.kind() == kind).count();
        }
        Self {
            row_count: rows.len(),
            counts,
        }
    }

    pub const fn row_count(&self) -> usize {
        self.row_count
    }

    pub const fn directional_adjacency_count(&self) -> usize {
        self.counts[0]
    }

    pub const fn reverse_adjacency_count(&self) -> usize {
        self.counts[1]
    }

    pub const fn predicate_support_count(&self) -> usize {
        self.counts[2]
    }

    pub const fn ordering_support_count(&self) -> usize {
        self.counts[3]
    }

    pub const fn traversal_workset_count(&self) -> usize {
        self.counts[4]
    }

    pub const fn visited_set_count(&self) -> usize {
        self.counts[5]
    }

    pub const fn dedup_set_count(&self) -> usize {
        self.counts[6]
    }

    pub const fn workset_count(&self) -> usize {
        self.counts[4] + self.counts[5] + self.counts[6]
    }

    pub const fn proof_support_count(&self) -> usize {
        self.counts[7]
    }

    pub const fn buffer_count(&self) -> usize {
        self.counts[8]
    }

    pub const fn materialization_lifecycle_count(&self) -> usize {
        self.counts[9]
    }

    pub const fn live_maintenance_support_count(&self) -> usize {
        self.counts[10]
    }

    pub const fn domain_operation_capability_registration_count(&self) -> usize {
        self.counts[11]
    }
}

fn derive_requirement_set_digest(
    read_graph_digest: CanonicalDigestId,
    access_shape_digest: CanonicalDigestId,
    selectivity_shape_digest: CanonicalDigestId,
    rows: &[WorthQueryGraphReadAccessRequirementRow],
    budget: CanonicalDigestWorkBudget,
) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial> {
    let mut basis = WorthQueryCanonicalIdentityBasis::new(
        canonical_basis::DOMAIN,
        canonical_basis::VERSION,
        budget,
    );
    canonical_basis::emit_fields(
        read_graph_digest,
        access_shape_digest,
        selectivity_shape_digest,
        rows,
        |field| match field {
            canonical_basis::RequirementDigestField::Digest(locus, value) => {
                basis.digest(locus, value)
            }
            canonical_basis::RequirementDigestField::Unsigned(locus, value) => {
                basis.unsigned(locus, value)
            }
            canonical_basis::RequirementDigestField::Row(index, row) => {
                let mut locus = String::new();
                canonical_basis::write_row_locus(&mut locus, index)
                    .expect("String formatting cannot fail");
                basis.text(locus, row.digest_part())
            }
        },
    )?;
    basis.derive()
}
