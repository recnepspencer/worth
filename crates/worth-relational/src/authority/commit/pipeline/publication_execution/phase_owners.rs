//! Exact installed owners selected for one candidate's publication preparation.
use super::PublicationRootCapture;
use crate::authority::commit::lineage_preparation::LineagePreparationAuthority;
use crate::execution::RequestWorkBudget;
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use crate::runtime::state::subsystems::indexing::IndexDefinitionReadBinding;
use crate::runtime::state::subsystems::storage::MirrorResidencyDemandBinding;
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::custom_registry::FrozenCustomInvariantRegistry;

pub(crate) struct PublicationPhaseOwners<'a, Attach> {
    pub(crate) root: PublicationRootCapture<'a, Attach>,
    pub(crate) work: Option<&'a RequestWorkBudget>,
    pub(crate) diagnostics: PreparationDiagnostics<'a>,
    pub(crate) lineage: LineagePreparationAuthority<'a>,
    pub(crate) fallback_schema: &'a RelationalSchemaRegistry,
    pub(crate) custom_invariants: &'a FrozenCustomInvariantRegistry,
    pub(crate) invariant_generation: u64,
    pub(crate) max_patch_records: Option<usize>,
    pub(crate) mirror: &'a MirrorResidencyDemandBinding,
    pub(crate) indexes: &'a IndexDefinitionReadBinding,
}
