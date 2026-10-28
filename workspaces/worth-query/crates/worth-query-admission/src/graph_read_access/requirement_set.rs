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
    directional_adjacency_count: usize,
    reverse_adjacency_count: usize,
    aggregate_selection_count: usize,
    predicate_support_count: usize,
    ordering_support_count: usize,
    traversal_workset_count: usize,
    visited_set_count: usize,
    dedup_set_count: usize,
    proof_support_count: usize,
    buffer_count: usize,
    materialization_lifecycle_count: usize,
    live_maintenance_support_count: usize,
    domain_operation_capability_registration_count: usize,
}

impl WorthQueryGraphReadAccessRequirementCounters {
    fn from_rows(rows: &[WorthQueryGraphReadAccessRequirementRow]) -> Self {
        let mut counts = Self {
            row_count: rows.len(),
            directional_adjacency_count: 0,
            reverse_adjacency_count: 0,
            aggregate_selection_count: 0,
            predicate_support_count: 0,
            ordering_support_count: 0,
            traversal_workset_count: 0,
            visited_set_count: 0,
            dedup_set_count: 0,
            proof_support_count: 0,
            buffer_count: 0,
            materialization_lifecycle_count: 0,
            live_maintenance_support_count: 0,
            domain_operation_capability_registration_count: 0,
        };
        for row in rows {
            // Exhaustive kind matching keeps new requirement kinds from silently
            // shifting existing metrics or overrunning a parallel ordinal array.
            match row.kind() {
                WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency => {
                    counts.directional_adjacency_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency => {
                    counts.reverse_adjacency_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::AggregateSelection => {
                    counts.aggregate_selection_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::PredicateSupport => {
                    counts.predicate_support_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::OrderingSupport => {
                    counts.ordering_support_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::TraversalWorkset => {
                    counts.traversal_workset_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::VisitedSet => {
                    counts.visited_set_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::DedupSet => counts.dedup_set_count += 1,
                WorthQueryGraphReadAccessRequirementKind::ProofSupport => {
                    counts.proof_support_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::ResultBuffer => counts.buffer_count += 1,
                WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle => {
                    counts.materialization_lifecycle_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport => {
                    counts.live_maintenance_support_count += 1
                }
                WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => {
                    counts.domain_operation_capability_registration_count += 1
                }
            }
        }
        counts
    }
    pub const fn row_count(&self) -> usize {
        self.row_count
    }
    pub const fn directional_adjacency_count(&self) -> usize {
        self.directional_adjacency_count
    }
    pub const fn reverse_adjacency_count(&self) -> usize {
        self.reverse_adjacency_count
    }
    pub const fn aggregate_selection_count(&self) -> usize {
        self.aggregate_selection_count
    }
    pub const fn predicate_support_count(&self) -> usize {
        self.predicate_support_count
    }
    pub const fn ordering_support_count(&self) -> usize {
        self.ordering_support_count
    }
    pub const fn traversal_workset_count(&self) -> usize {
        self.traversal_workset_count
    }
    pub const fn visited_set_count(&self) -> usize {
        self.visited_set_count
    }
    pub const fn dedup_set_count(&self) -> usize {
        self.dedup_set_count
    }
    pub const fn proof_support_count(&self) -> usize {
        self.proof_support_count
    }
    pub const fn buffer_count(&self) -> usize {
        self.buffer_count
    }
    pub const fn materialization_lifecycle_count(&self) -> usize {
        self.materialization_lifecycle_count
    }
    pub const fn live_maintenance_support_count(&self) -> usize {
        self.live_maintenance_support_count
    }
    pub const fn domain_operation_capability_registration_count(&self) -> usize {
        self.domain_operation_capability_registration_count
    }
    pub const fn workset_count(&self) -> usize {
        self.traversal_workset_count + self.visited_set_count + self.dedup_set_count
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
