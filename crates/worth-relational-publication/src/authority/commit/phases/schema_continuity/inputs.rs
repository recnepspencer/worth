//! Exact configuration, history and original schema consent for one continuity decision.
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use contracts::config::data::sections::SchemaConfig;
use contracts::schema::data::continuity::transition_classification::SchemaReconciliationPolicy;
use contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::schema::continuity::authority_input::SchemaContinuityAuthorityInput;
use worth_relational_contracts::internal_port as contracts;

pub struct SchemaContinuityInputs<'a> {
    pub schema: &'a SchemaConfig,
    pub history: &'a RelationalPreparationHistory,
    pub diagnostics: PreparationDiagnostics<'a>,
}

pub struct SchemaContinuityRequest<'a> {
    pub basis: &'a AdmittedRelationalBranchBasis,
    pub authority: &'a RelationalBranchRootSchemaAuthority,
    pub copied_authority: Option<&'a SchemaContinuityAuthorityInput>,
    pub transition: Option<&'a ProposedSchemaTransition>,
    pub reconciliation: Option<&'a SchemaReconciliationPolicy>,
}
