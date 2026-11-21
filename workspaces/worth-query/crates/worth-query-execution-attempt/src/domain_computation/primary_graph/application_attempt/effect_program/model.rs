use std::marker::PhantomData;
use worth_relational::facade::transactions::EntityReference;
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
    pub producer_round_limits: (std::num::NonZeroUsize, std::num::NonZeroUsize),
    pub authored_stages: Vec<crate::ports::OpaqueAuthoredEditStage>,
    pub synchronous_outputs: Vec<crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::CapturedSynchronousOutputs>,
    pub read_set:
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
    pub effects:
        Vec<WorthQueryApplicationRealizedEffect>,
    pub conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    pub effect_posture:
        crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture,
    pub output_correspondence:
        super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    pub retain_output_demand_observation: bool,
    pub retain_client_observation: bool,
    pub producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    pub output_currentness_facts:
        Option<super::super::OutputCurrentnessFacts>,
    pub content_attachments:
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
    pub read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    pub layout: Arc<super::super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    pub program: Arc<()>,
    pub effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub field_write_positions: super::FieldWriteIndex,
    pub case_transitions: super::case_transition::CaseTransitions,
    pub keys: BTreeSet<(KindId, String)>,
    pub conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    pub output_correspondence:
        super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    pub creation_partition: Option<WorthQueryApplicationCreationPartition>,
    pub content_attachments: worth_relational::facade::content::RelationalContentAttachments,
    pub content_denial: Option<crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial>,
    // Last to drop: physical backing releases before its retained grant.

}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub fn refuse_stale_home_authority(
        &self,
        current: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        self.read_set.admission.refuse_stale_home_authority(current)
    }
    pub fn belongs_to_application(
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

    pub fn with_client_observation(mut self) -> Self {
        self.retain_client_observation = true;
        self
    }

    pub fn with_producer_required_invariants(
        mut self,
        requirements: &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    ) -> Self {
        self.producer_required_invariants = requirements;
        self
    }

    pub fn with_authored_edit_stages<Stage: Into<crate::ports::OpaqueAuthoredEditStage>>(
        mut self,
        _access: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
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

