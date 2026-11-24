//! Exact configuration, history and original schema consent for one continuity decision.
use crate::branch::{
    basis::AdmittedRelationalBranchBasis, root::schema::RelationalBranchRootSchemaAuthority,
};
use crate::config::data::SchemaConfig;
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};
use crate::schema::SchemaContinuityAuthorityInput;

pub(crate) struct SchemaContinuityInputs<'a> {
    pub(crate) schema: &'a SchemaConfig,
    pub(crate) history: &'a RelationalPreparationHistory,
    pub(crate) diagnostics: PreparationDiagnostics<'a>,
}

pub(crate) struct SchemaContinuityRequest<'a> {
    pub(crate) basis: &'a AdmittedRelationalBranchBasis,
    pub(crate) authority: &'a RelationalBranchRootSchemaAuthority,
    pub(crate) copied_authority: Option<&'a SchemaContinuityAuthorityInput>,
    pub(crate) transition: Option<&'a ProposedSchemaTransition>,
    pub(crate) reconciliation: Option<&'a SchemaReconciliationPolicy>,
}
