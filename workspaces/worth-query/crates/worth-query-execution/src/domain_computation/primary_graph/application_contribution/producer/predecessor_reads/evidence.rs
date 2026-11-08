//! Descriptive old-read provenance; it owns neither a root nor current dependencies.
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use std::{collections::BTreeSet, sync::Arc};
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
pub(in crate::domain_computation::primary_graph) struct HistoricalProducerRead {
    pub(in crate::domain_computation::primary_graph) publication: (u64, usize),
    pub(in crate::domain_computation::primary_graph) before_basis: RelationalBranchBasisDescriptor,
    pub(in crate::domain_computation::primary_graph) applied:
        Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    pub(in crate::domain_computation::primary_graph) operation: &'static str,
    pub(in crate::domain_computation::primary_graph) keys:
        Arc<BTreeSet<WorthQueryApplicationFactKey>>,
}
