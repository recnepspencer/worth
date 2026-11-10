//! Descriptive old-read provenance; it owns neither a root nor current dependencies.
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use std::{collections::BTreeSet, sync::Arc};
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
pub(in crate::domain_computation::primary_graph) struct HistoricalProducerRead {
    pub(in crate::domain_computation::primary_graph) _publication: (u64, usize),
    pub(in crate::domain_computation::primary_graph) _before_basis: RelationalBranchBasisDescriptor,
    pub(in crate::domain_computation::primary_graph) _applied:
        Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    pub(in crate::domain_computation::primary_graph) _operation: &'static str,
    pub(in crate::domain_computation::primary_graph) _keys:
        Arc<BTreeSet<WorthQueryApplicationFactKey>>,
}
