//! Crate-boundary ports exposing owner-private `WorthQueryApplicationEffectProgram`
//! and `WorthQueryApplicationEffectProgramBuilder` internals strictly to upper
//! execution split crates. Never exported on `worth-query-execution::facade` or Host.

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::Arc;

use worth_relational::facade::content::RelationalContentAttachments;
use worth_relational::facade::identity::KindId;

use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    model::{WorthQueryApplicationCreationPartition, WorthQueryApplicationRealizedEffect},
    output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationEffectProgramBuilder,
};
use crate::domain_computation::primary_graph::application_attempt::read_set::WorthQueryCompleteApplicationReadSet;
use crate::domain_computation::primary_graph::application_attempt::{
    OutputCurrentnessFacts, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::CapturedSynchronousOutputs;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationConditionalDefinition, WorthQueryProducerInvariantRequirement,
    WorthQueryValueDecodeDenial,
};
use crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture;
use crate::ports::OpaqueAuthoredEditStage;

/// Unpacked internal fields of a `WorthQueryApplicationEffectProgram` for upper
/// execution split crates only.
pub struct EffectProgramParts<Schema, Operation, Input, Scope> {
    pub producer_round_limits: (NonZeroUsize, NonZeroUsize),
    pub authored_stages: Vec<OpaqueAuthoredEditStage<Schema>>,
    pub synchronous_outputs: Vec<CapturedSynchronousOutputs>,
    pub read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    pub effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub conditional_definition: Option<WorthQueryAdmittedApplicationConditionalDefinition>,
    pub effect_posture: WorthQueryApplicationEffectPosture,
    pub output_correspondence: WorthQueryApplicationOutputCorrespondenceCandidate,
    pub persistent_names: worth_query_execution_lineage::facade::persistent_naming::AuthoredOperationNaming,
    pub retain_output_demand_observation: bool,
    pub retain_client_observation: bool,
    pub producer_required_invariants: &'static [WorthQueryProducerInvariantRequirement],
    pub output_currentness_facts: Option<OutputCurrentnessFacts>,
    pub content_attachments: RelationalContentAttachments,
}

pub trait EffectProgramInternalPort<Schema, Operation, Input, Scope>: Sized {
    fn from_parts(parts: EffectProgramParts<Schema, Operation, Input, Scope>) -> Self;

    fn into_parts(self) -> EffectProgramParts<Schema, Operation, Input, Scope>;

    fn read_set(
        &self,
    ) -> &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >;

    fn read_set_mut(
        &mut self,
    ) -> &mut WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >;

    fn effects(&self) -> &[WorthQueryApplicationRealizedEffect];

    fn effects_mut(&mut self) -> &mut Vec<WorthQueryApplicationRealizedEffect>;

    fn authored_stages(&self) -> &[OpaqueAuthoredEditStage<Schema>];

    fn synchronous_outputs(&self) -> &[CapturedSynchronousOutputs];

    fn synchronous_outputs_mut(&mut self) -> &mut Vec<CapturedSynchronousOutputs>;

    fn producer_round_limits(&self) -> (NonZeroUsize, NonZeroUsize);

    fn set_producer_round_limits(&mut self, limits: (NonZeroUsize, NonZeroUsize));

    fn conditional_definition(&self)
        -> Option<&WorthQueryAdmittedApplicationConditionalDefinition>;

    fn effect_posture(&self) -> WorthQueryApplicationEffectPosture;

    fn set_effect_posture(&mut self, posture: WorthQueryApplicationEffectPosture);

    fn output_correspondence(&self) -> &WorthQueryApplicationOutputCorrespondenceCandidate;

    fn output_currentness_facts(&self) -> Option<&OutputCurrentnessFacts>;

    fn set_output_currentness_facts(&mut self, facts: Option<OutputCurrentnessFacts>);

    fn retain_output_demand_observation(&self) -> bool;

    fn or_retain_output_demand_observation(&mut self, flag: bool);

    fn retain_client_observation(&self) -> bool;

    fn or_retain_client_observation(&mut self, flag: bool);

    fn producer_required_invariants(&self) -> &'static [WorthQueryProducerInvariantRequirement];

    fn content_attachments(&self) -> &RelationalContentAttachments;
}

pub trait EffectProgramBuilderInternalPort<Schema, Operation, Input, Scope> {
    fn read_set(
        &self,
    ) -> &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >;

    fn layout(&self) -> &Arc<WorthQueryPrimaryGraphLayout>;

    fn program_token(&self) -> &Arc<()>;

    fn effects(&self) -> &[WorthQueryApplicationRealizedEffect];

    fn effects_mut(&mut self) -> &mut Vec<WorthQueryApplicationRealizedEffect>;

    fn keys_mut(&mut self) -> &mut BTreeSet<(KindId, String)>;

    fn output_correspondence_mut(
        &mut self,
    ) -> &mut WorthQueryApplicationOutputCorrespondenceCandidate;

    fn creation_partition(&self) -> Option<WorthQueryApplicationCreationPartition>;

    fn content_attachments_mut(&mut self) -> &mut RelationalContentAttachments;

    fn content_denial_mut(&mut self) -> &mut Option<WorthQueryValueDecodeDenial>;
}

impl<Schema, Operation, Input, Scope> EffectProgramInternalPort<Schema, Operation, Input, Scope>
    for WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    fn from_parts(parts: EffectProgramParts<Schema, Operation, Input, Scope>) -> Self {
        Self {
            producer_round_limits: parts.producer_round_limits,
            authored_stages: parts.authored_stages,
            synchronous_outputs: parts.synchronous_outputs,
            read_set: parts.read_set,
            effects: parts.effects,
            conditional_definition: parts.conditional_definition,
            effect_posture: parts.effect_posture,
            output_correspondence: parts.output_correspondence,
            persistent_names: parts.persistent_names,
            retain_output_demand_observation: parts.retain_output_demand_observation,
            retain_client_observation: parts.retain_client_observation,
            producer_required_invariants: parts.producer_required_invariants,
            output_currentness_facts: parts.output_currentness_facts,
            content_attachments: parts.content_attachments,
        }
    }

    fn into_parts(self) -> EffectProgramParts<Schema, Operation, Input, Scope> {
        EffectProgramParts {
            producer_round_limits: self.producer_round_limits,
            authored_stages: self.authored_stages,
            synchronous_outputs: self.synchronous_outputs,
            read_set: self.read_set,
            effects: self.effects,
            conditional_definition: self.conditional_definition,
            effect_posture: self.effect_posture,
            output_correspondence: self.output_correspondence,
            persistent_names: self.persistent_names,
            retain_output_demand_observation: self.retain_output_demand_observation,
            retain_client_observation: self.retain_client_observation,
            producer_required_invariants: self.producer_required_invariants,
            output_currentness_facts: self.output_currentness_facts,
            content_attachments: self.content_attachments,
        }
    }

    fn read_set(
        &self,
    ) -> &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        &self.read_set
    }

    fn read_set_mut(
        &mut self,
    ) -> &mut WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        &mut self.read_set
    }

    fn effects(&self) -> &[WorthQueryApplicationRealizedEffect] {
        &self.effects
    }

    fn effects_mut(&mut self) -> &mut Vec<WorthQueryApplicationRealizedEffect> {
        &mut self.effects
    }

    fn authored_stages(&self) -> &[OpaqueAuthoredEditStage<Schema>] {
        &self.authored_stages
    }

    fn synchronous_outputs(&self) -> &[CapturedSynchronousOutputs] {
        &self.synchronous_outputs
    }

    fn synchronous_outputs_mut(&mut self) -> &mut Vec<CapturedSynchronousOutputs> {
        &mut self.synchronous_outputs
    }

    fn producer_round_limits(&self) -> (NonZeroUsize, NonZeroUsize) {
        self.producer_round_limits
    }

    fn set_producer_round_limits(&mut self, limits: (NonZeroUsize, NonZeroUsize)) {
        self.producer_round_limits = limits;
    }

    fn conditional_definition(
        &self,
    ) -> Option<&WorthQueryAdmittedApplicationConditionalDefinition> {
        self.conditional_definition.as_ref()
    }

    fn effect_posture(&self) -> WorthQueryApplicationEffectPosture {
        self.effect_posture
    }

    fn set_effect_posture(&mut self, posture: WorthQueryApplicationEffectPosture) {
        self.effect_posture = posture;
    }

    fn output_correspondence(&self) -> &WorthQueryApplicationOutputCorrespondenceCandidate {
        &self.output_correspondence
    }

    fn output_currentness_facts(&self) -> Option<&OutputCurrentnessFacts> {
        self.output_currentness_facts.as_ref()
    }

    fn set_output_currentness_facts(&mut self, facts: Option<OutputCurrentnessFacts>) {
        self.output_currentness_facts = facts;
    }

    fn retain_output_demand_observation(&self) -> bool {
        self.retain_output_demand_observation
    }

    fn or_retain_output_demand_observation(&mut self, flag: bool) {
        self.retain_output_demand_observation |= flag;
    }

    fn retain_client_observation(&self) -> bool {
        self.retain_client_observation
    }

    fn or_retain_client_observation(&mut self, flag: bool) {
        self.retain_client_observation |= flag;
    }

    fn producer_required_invariants(&self) -> &'static [WorthQueryProducerInvariantRequirement] {
        self.producer_required_invariants
    }

    fn content_attachments(&self) -> &RelationalContentAttachments {
        &self.content_attachments
    }
}

impl<Schema, Operation, Input, Scope>
    EffectProgramBuilderInternalPort<Schema, Operation, Input, Scope>
    for WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    fn read_set(
        &self,
    ) -> &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        &self.read_set
    }

    fn layout(&self) -> &Arc<WorthQueryPrimaryGraphLayout> {
        &self.layout
    }

    fn program_token(&self) -> &Arc<()> {
        &self.program
    }

    fn effects(&self) -> &[WorthQueryApplicationRealizedEffect] {
        &self.effects
    }

    fn effects_mut(&mut self) -> &mut Vec<WorthQueryApplicationRealizedEffect> {
        &mut self.effects
    }

    fn keys_mut(&mut self) -> &mut BTreeSet<(KindId, String)> {
        &mut self.keys
    }

    fn output_correspondence_mut(
        &mut self,
    ) -> &mut WorthQueryApplicationOutputCorrespondenceCandidate {
        &mut self.output_correspondence
    }

    fn creation_partition(&self) -> Option<WorthQueryApplicationCreationPartition> {
        self.creation_partition
    }

    fn content_attachments_mut(&mut self) -> &mut RelationalContentAttachments {
        &mut self.content_attachments
    }

    fn content_denial_mut(&mut self) -> &mut Option<WorthQueryValueDecodeDenial> {
        &mut self.content_denial
    }
}
