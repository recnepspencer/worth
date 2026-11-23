use std::collections::BTreeSet;
use std::sync::Arc;

use worth_relational::facade::identity::KindId;

use super::super::read_set::WorthQueryCompleteApplicationReadSet;
use super::super::WorthQueryProjectedApplicationMutation;

pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::effect_program::model::{
    emission, WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationCreatedRelation,
    WorthQueryApplicationCreationPartition, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEmission, WorthQueryApplicationOptionalFieldWrite,
    WorthQueryApplicationRealizedEffect,
};

/// A finished effect program: the sealed read set plus the effects authored
/// against it, ready to compare and commit.
///
/// Built by [`WorthQueryApplicationEffectProgramBuilder`]'s `finish`, which
/// re-checked current authority and the output correspondence. Hand it to the
/// application runtime's `compare_and_commit_application` with an idempotency
/// binding. Building it changed nothing.
pub struct WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
    pub(crate) producer_round_limits: (std::num::NonZeroUsize, std::num::NonZeroUsize),
    pub(crate) authored_stages: Vec<crate::ports::OpaqueAuthoredEditStage<Schema>>,
    pub(crate) synchronous_outputs: Vec<crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::CapturedSynchronousOutputs>,
    pub(crate) read_set:
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
    pub(crate) effects:
        Vec<WorthQueryApplicationRealizedEffect>,
    pub(crate) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    pub(crate) effect_posture:
        crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture,
    pub(crate) output_correspondence:
        super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    /// Declared persistent-name births and postures over this program's
    /// outputs; empty for every unnamed program.
    pub(crate) persistent_names: worth_query_execution_lineage::facade::persistent_naming::AuthoredOperationNaming,
    pub(crate) retain_output_demand_observation: bool,
    pub(crate) retain_client_observation: bool,
    pub(crate) producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    pub(crate) output_currentness_facts:
        Option<super::super::OutputCurrentnessFacts>,
    pub(crate) content_attachments:
        worth_relational::facade::content::RelationalContentAttachments,
    // Last to drop: physical backing releases before its retained grant.

}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    /// Original host controls, carried as facts rather than execution custody.
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.read_set.admission.publication_request()
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub fn with_consumed_output_for_test(
        mut self,
        evidence: crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence,
    ) -> Self {
        self.read_set.consumed_outputs.push(evidence);
        self
    }
}

/// Authors the effects of one candidate against a complete projected read set.
///
/// Begin with `begin_effect_program`. Each write, create, delete, link, unlink,
/// or emit is checked against the operation's installed authority. Call `finish` to get the [`WorthQueryApplicationEffectProgram`].
pub struct WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope> {
    pub(crate) read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    pub(crate) layout: Arc<super::super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    pub(crate) program: Arc<()>,
    pub(crate) effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub(crate) field_write_positions: super::FieldWriteIndex,
    pub(crate) case_transitions: super::case_transition::CaseTransitions,
    pub(crate) keys: BTreeSet<(KindId, String)>,
    pub(crate) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    pub(crate) output_correspondence:
        super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    pub(crate) persistent_names: worth_query_execution_lineage::facade::persistent_naming::AuthoredOperationNaming,
    pub(crate) creation_partition: Option<WorthQueryApplicationCreationPartition>,
    pub(crate) content_attachments: worth_relational::facade::content::RelationalContentAttachments,
    pub(crate) content_denial: Option<crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial>,
    // Last to drop: physical backing releases before its retained grant.

}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(crate) fn refuse_stale_home_authority(
        &self,
        current: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        self.read_set.admission.refuse_stale_home_authority(current)
    }
    pub(crate) fn belongs_to_application(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.read_set
            .admission
            .belongs_to(runtime_authority, binding_identity)
    }

    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.read_set.lease.product().product_branch()
    }

    pub fn with_output_demand_observation(mut self) -> Self {
        self.retain_output_demand_observation = true;
        self
    }

    pub(crate) fn with_client_observation(mut self) -> Self {
        self.retain_client_observation = true;
        self
    }

    pub(crate) fn with_producer_required_invariants(
        mut self,
        requirements: &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    ) -> Self {
        self.producer_required_invariants = requirements;
        self
    }

    pub fn with_authored_edit_stages<Stage: Into<crate::ports::OpaqueAuthoredEditStage<Schema>>>(
        mut self,
        _access: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        stages: Vec<Stage>,
    ) -> Self {
        self.authored_stages = stages.into_iter().map(Into::into).collect();
        self
    }
}

pub mod court_doors_for_test;

#[cfg(test)]
#[path = "model_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

// Capturing synchronous work must preserve the ordinary program's Send contract.
const _: fn() = || {
    fn send<T: Send>() {}
    send::<WorthQueryApplicationEffectProgram<(), (), u8, ()>>();
};
