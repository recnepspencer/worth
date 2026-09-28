use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, PortableAspectContractBasis};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::transactions::EntityReference;

use super::super::read_set::WorthQueryCompleteApplicationReadSet;
use super::super::WorthQueryProjectedApplicationMutation;

mod emission;
pub(in crate::domain_computation::primary_graph) use emission::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission,
};

/// An entity handle usable only inside the effect program that produced it: an
/// existing entity from the read set, or one the program creates.
///
/// Get one from the builder's `existing_entity`, `projected_entity`, or
/// `create_entity`, then write fields or link relations through it. A handle
/// from another effect program is refused.
pub struct WorthQueryApplicationEffectEntity<Schema, Entity> {
    pub(super) reference: EntityReference,
    pub(super) entity: String,
    pub(super) created_effect: Option<usize>,
    pub(super) program: Arc<()>,
    pub(super) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationOptionalFieldWrite {
    pub(in crate::domain_computation::primary_graph::application_attempt) contract:
        PortableAspectContractBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) value:
        Option<AspectValue>,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryApplicationRealizedEffect {
    CreateEntity {
        kind: KindId,
        key: String,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
        partition: WorthQueryApplicationCreationPartition,
    },
    UpdateEntity {
        entity: String,
        entity_id: EntityId,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
    },
    PatchOptionalEntityFields {
        entity: String,
        entity_id: EntityId,
        fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
    },
    DeleteEntity {
        entity_id: EntityId,
    },
    CreateRelation {
        kind: KindId,
        key: String,
        from: EntityReference,
        to: EntityReference,
    },
    DeleteRelation {
        relation_id: RelationId,
    },
    Emit(WorthQueryApplicationEmission),
}

/// A finished effect program: the sealed read set plus the effects authored
/// against it, ready to compare and commit.
///
/// Built by [`WorthQueryApplicationEffectProgramBuilder`]'s `finish`, which
/// re-checked current authority and the output correspondence. Hand it to the
/// application runtime's `compare_and_commit_application` with an idempotency
/// binding. Building it changed nothing.
pub struct WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph::application_attempt) read_set:
        WorthQueryCompleteApplicationReadSet<
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
    pub(in crate::domain_computation::primary_graph::application_attempt) effects:
        Vec<WorthQueryApplicationRealizedEffect>,
    pub(in crate::domain_computation::primary_graph::application_attempt) emission_retained_bytes:
        u64,
    pub(in crate::domain_computation::primary_graph::application_attempt) emission_retained_bytes_ceiling:
        u64,
    pub(in crate::domain_computation::primary_graph::application_attempt) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    /// Whether effects follow the application contract, Query's reserved
    /// platform contract, or both within one candidate.
    pub(in crate::domain_computation::primary_graph::application_attempt) effect_posture:
        crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture,
    pub(in crate::domain_computation::primary_graph::application_attempt) validator_work_admission:
        super::WorthQueryCandidateValidatorWorkAdmission,
    pub(in crate::domain_computation::primary_graph::application_attempt) output_correspondence:
        super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    pub(in crate::domain_computation::primary_graph::application_attempt) retain_output_demand_observation:
        bool,
    pub(in crate::domain_computation::primary_graph::application_attempt) retain_client_observation:
        bool,
    pub(in crate::domain_computation::primary_graph::application_attempt) producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    pub(in crate::domain_computation::primary_graph::application_attempt) output_currentness_facts:
        Option<Arc<[super::super::WorthQueryApplicationObservedFact]>>,
}

/// Authors the effects of one candidate against a complete projected read set.
///
/// Begin with `begin_effect_program`. Each write, create, delete, link, unlink,
/// or emit is checked against the operation's installed ceiling and charged to
/// its candidate reservation, and is refused with a typed attempt denial when it
/// does not fit. Call `finish` to get the [`WorthQueryApplicationEffectProgram`].
pub struct WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope> {
    pub(super) read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    pub(super) layout: Arc<super::super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    pub(super) program: Arc<()>,
    pub(super) effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub(super) field_write_positions: super::FieldWriteIndex,
    pub(super) keys: BTreeSet<(KindId, String)>,
    pub(super) emission_retained_bytes: u64,
    pub(super) emission_retained_bytes_ceiling: u64,
    pub(super) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    pub(super) candidate_reservation: Option<super::WorthQueryCandidateReservation>,
    pub(super) output_correspondence:
        super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    pub(super) creation_partition: Option<WorthQueryApplicationCreationPartition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryApplicationCreationPartition {
    Issued,
    Context(worth_relational::facade::identity::PartitionId),
}

impl WorthQueryApplicationCreationPartition {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn resolve(
        self,
        issued: worth_relational::facade::identity::PartitionId,
    ) -> worth_relational::facade::identity::PartitionId {
        match self {
            Self::Issued => issued,
            Self::Context(partition) => partition,
        }
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn belongs_to_application(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.read_set
            .admission
            .belongs_to(runtime_authority, binding_identity)
    }

    pub(in crate::domain_computation::primary_graph) fn product_branch(
        &self,
    ) -> crate::basis::WorthQueryProductBranch {
        self.read_set.lease.product().product_branch()
    }

    pub(in crate::domain_computation::primary_graph) fn with_output_demand_observation(
        mut self,
    ) -> Self {
        self.retain_output_demand_observation = true;
        self
    }

    pub(in crate::domain_computation::primary_graph) fn with_client_observation(mut self) -> Self {
        self.retain_client_observation = true;
        self
    }

    pub(in crate::domain_computation::primary_graph) fn with_producer_required_invariants(
        mut self,
        requirements: &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    ) -> Self {
        self.producer_required_invariants = requirements;
        self
    }
}
