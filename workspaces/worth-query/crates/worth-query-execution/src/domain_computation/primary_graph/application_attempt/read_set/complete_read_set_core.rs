//! Read facts sealed by completion, before the original brand moves on.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::CompletedReadScopes;
use crate::domain_computation::primary_graph::SealedComputationRetention;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::application_projection::CompletedFactProjection;

// Only read completion constructs this data. The admission and phase brand
// never leave the shell; this value cannot mint or rebrand their authority.
pub(super) struct CompleteReadSetCore {
    pub(super) read: CompletedReadFacts,
    pub(super) effect_routes: super::super::CompletedFactRoutes,
}

pub(super) struct CompletedReadFacts {
    pub(super) application_projection: Option<Arc<CompletedFactProjection>>,
    pub(super) original_read: Option<Arc<CompletedOriginalRead>>,
    pub(super) installed_read_scopes: CompletedReadScopes,
    pub(super) facts: FactPayload,
    pub(super) content: Option<super::super::FactContent>,
    pub(super) postcondition_carry:
        Option<super::super::super::output_lineage::invalidation::PostconditionCarry>,
    pub(super) computation_facts: Option<SealedComputationFacts>,
    pub(super) computation_fact_ordinals: Arc<[usize]>,
}

pub(super) struct NativeReadCompletion {
    pub(super) read: CompletedReadFacts,
    pub(super) retained: SealedComputationRetention,
    pub(super) handler_count: usize,
    pub(super) content_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

pub(super) enum PreparedReadCompletion {
    Full(CompleteReadSetCore),
    Native(NativeReadCompletion),
}
