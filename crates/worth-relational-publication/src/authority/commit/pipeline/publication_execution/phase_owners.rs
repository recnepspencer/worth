//! Exact installed owners selected for one candidate's publication preparation.
use super::PublicationRootCapture;
use crate::authority::commit::lineage_preparation::LineagePreparationAuthority;
use worth_relational_contracts::internal_port::execution::request_work::RequestWorkBudget;
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexDefinitionReadBinding;
use worth_relational_storage::internal_port::runtime::state::subsystems::storage::residency_demand::MirrorResidencyDemandBinding;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistry;

pub struct PublicationPhaseOwners<'a, Attach> {
    pub root: PublicationRootCapture<'a, Attach>,
    pub work: Option<&'a RequestWorkBudget>,
    pub diagnostics: PreparationDiagnostics<'a>,
    pub lineage: LineagePreparationAuthority<'a>,
    pub fallback_schema: &'a RelationalSchemaRegistry,
    pub invariant_generation: u64,
    pub max_patch_records: Option<usize>,
    pub mirror: &'a MirrorResidencyDemandBinding,
    pub indexes: &'a IndexDefinitionReadBinding,
}
