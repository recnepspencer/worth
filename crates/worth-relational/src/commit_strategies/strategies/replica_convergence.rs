use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey};

use crate::commit_strategies::data::{
    decode_entity_id, encode_entity_id, encode_u64, CanonicalStrategyCommitRequest,
    CanonicalStrategyOutputArtifact, CommitStrategyDescriptor, CommitStrategyExecutionRegistration,
    CommitStrategyExecutor, CommitStrategyFamilyName, CommitStrategyId, CommitStrategyRegistration,
    CommitStrategyRegistrationError, CommitStrategySemanticName, CommitStrategyVersion,
    NativeCodecError, NativeCodecReader, NativeStrategyCommitRequest, PersistentArtifactName,
    StrategyCallerProvenance, StrategyEntityAspectReadRecord, StrategyExecutionResult,
    StrategyExecutorFailure, StrategyExecutorFailureClass, StrategyInputSchemaName,
    StrategyInputSchemaVersion, StrategyIntentName, StrategyMutationProgram,
    StrategyObservationContext, StrategyOutputSchemaName, StrategyPacketContract,
    StrategyReadContract, StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
    StrategyTraversalBasis,
};
use crate::identity::data::EntityId;
use crate::storage::data::authoritative_aspect_value_field_comparison_key;
use crate::transactions::data::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaConvergenceInput {
    pub entity_id: EntityId,
    pub desired_replicas: u64,
}

impl ReplicaConvergenceInput {
    pub fn into_native_canonical_request(
        self,
        caller_provenance: StrategyCallerProvenance,
    ) -> Result<NativeStrategyCommitRequest, NativeCodecError> {
        let mut bytes = Vec::new();
        encode_entity_id(&mut bytes, self.entity_id);
        encode_u64(&mut bytes, self.desired_replicas);
        Ok(NativeStrategyCommitRequest::from_native_canonical_bytes(
            CommitStrategySemanticName::new(ReplicaConvergenceStrategy::DEFAULT_SEMANTIC_NAME),
            bytes,
            caller_provenance,
        ))
    }

    fn decode(bytes: &[u8]) -> Result<Self, NativeCodecError> {
        let mut reader = NativeCodecReader::new(bytes);
        let entity_id = decode_entity_id(&mut reader)?;
        let desired_replicas = reader.read_u64()?;
        reader.finish()?;
        Ok(Self {
            entity_id,
            desired_replicas,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicaConvergenceAction {
    NoChange,
    UpdateReplicas,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaConvergenceOutput {
    pub entity_id: EntityId,
    pub action: ReplicaConvergenceAction,
    pub desired_replicas: u64,
}

struct PlannedReplicaFieldPatch {
    target: AspectFieldLocator,
    desired_value: AspectValue,
    fields: AspectFieldPatch,
}

impl ReplicaConvergenceOutput {
    pub fn decode(bytes: &[u8]) -> Result<Self, NativeCodecError> {
        let mut reader = NativeCodecReader::new(bytes);
        let entity_id = decode_entity_id(&mut reader)?;
        let action = match reader.read_u8()? {
            0 => ReplicaConvergenceAction::NoChange,
            1 => ReplicaConvergenceAction::UpdateReplicas,
            tag => {
                return Err(NativeCodecError::new(format!(
                    "unknown replica convergence action tag {tag}"
                )))
            }
        };
        let desired_replicas = reader.read_u64()?;
        reader.finish()?;
        Ok(Self {
            entity_id,
            action,
            desired_replicas,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ReplicaConvergenceStrategy;

impl ReplicaConvergenceStrategy {
    pub const DEFAULT_SEMANTIC_NAME: &'static str = "strategy.replica.converge";
    pub const DEFAULT_FAMILY_NAME: &'static str = "strategy.replica";
    pub const DEFAULT_INPUT_SCHEMA_NAME: &'static str = "replica.converge.input.v1";
    pub const DEFAULT_OUTPUT_SCHEMA_NAME: &'static str = "replica.converge.output.v1";
    pub const DEFAULT_INTENT_NAME: &'static str = "replica.desired.state";
    pub const DEFAULT_ARTIFACT_NAME: &'static str = "strategy.replica.converge";

    pub fn descriptor(strategy_id: CommitStrategyId) -> CommitStrategyDescriptor {
        CommitStrategyDescriptor::new(
            strategy_id,
            CommitStrategySemanticName::new(Self::DEFAULT_SEMANTIC_NAME),
            CommitStrategyFamilyName::new(Self::DEFAULT_FAMILY_NAME),
            CommitStrategyVersion::new(1, 0),
            StrategyIntentName::new(Self::DEFAULT_INTENT_NAME),
            StrategyInputSchemaName::new(Self::DEFAULT_INPUT_SCHEMA_NAME),
            StrategyInputSchemaVersion(1),
            StrategyOutputSchemaName::new(Self::DEFAULT_OUTPUT_SCHEMA_NAME),
            StrategyReadContract {
                scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
                locality_class: StrategyReadLocalityClass::SinglePartition,
                traversal_basis: StrategyTraversalBasis::NoTraversal,
                packet_contract: StrategyPacketContract::ProjectionOnly,
                cost_class: StrategyReadCostClass::ORequestedSurface,
            },
            PersistentArtifactName::new(Self::DEFAULT_ARTIFACT_NAME),
        )
    }

    pub fn registration(
        strategy_id: CommitStrategyId,
    ) -> Result<CommitStrategyRegistration, CommitStrategyRegistrationError> {
        CommitStrategyRegistration::new(Self::descriptor(strategy_id))
    }

    pub fn execution_registration(
        descriptor: &CommitStrategyDescriptor,
    ) -> CommitStrategyExecutionRegistration {
        CommitStrategyExecutionRegistration::new(descriptor, Self)
    }

    fn parse_input(
        request: &CanonicalStrategyCommitRequest,
    ) -> Result<ReplicaConvergenceInput, StrategyExecutorFailure> {
        ReplicaConvergenceInput::decode(request.canonical_input().canonical_bytes()).map_err(
            |error| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::InvalidInput,
                    format!(
                        "replica convergence input could not be decoded: {}",
                        error.detail()
                    ),
                )
            },
        )
    }

    fn output_artifact(output: &ReplicaConvergenceOutput) -> CanonicalStrategyOutputArtifact {
        let mut bytes = Vec::new();
        encode_entity_id(&mut bytes, output.entity_id);
        bytes.push(match output.action {
            ReplicaConvergenceAction::NoChange => 0,
            ReplicaConvergenceAction::UpdateReplicas => 1,
        });
        encode_u64(&mut bytes, output.desired_replicas);
        CanonicalStrategyOutputArtifact::new(
            StrategyOutputSchemaName::new(Self::DEFAULT_OUTPUT_SCHEMA_NAME),
            bytes,
            PersistentArtifactName::new(Self::DEFAULT_ARTIFACT_NAME),
        )
    }

    fn require_lowered_entity_scalar_field(
        observation: &StrategyObservationContext<'_>,
        existing: &StrategyEntityAspectReadRecord,
        field: &FieldKey,
    ) -> Result<worth_foundational::facade::AspectKey, StrategyExecutorFailure> {
        let lowered_plan = observation
            .entity_aspect_plan(existing.kind_id())
            .ok_or_else(|| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::DomainRejection,
                    format!(
                        "lowered foundational entity aspect plan is missing for kind {} during replica convergence",
                        existing.kind_name()
                    ),
                )
            })?;
        lowered_plan
            .entity_scalar_field_aspect_key(field)
            .ok_or_else(|| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::DomainRejection,
                    format!(
                        "field '{}' is not a lowered foundational scalar entity aspect on kind {}",
                        field.as_str(),
                        existing.kind_name()
                    ),
                )
            })
    }

    fn plan_replica_field_patch(
        observation: &StrategyObservationContext<'_>,
        existing: &StrategyEntityAspectReadRecord,
        desired_replicas: u64,
    ) -> Result<PlannedReplicaFieldPatch, StrategyExecutorFailure> {
        let replicas = FieldKey::new("replicas").expect("static field key must be valid");
        let aspect_key =
            Self::require_lowered_entity_scalar_field(observation, existing, &replicas)?;
        let target = crate::transactions::data::planned_single_field_locator(aspect_key, replicas);
        let desired_value = AspectValue::UInt64(desired_replicas);
        let fields =
            AspectFieldPatch::new(BTreeMap::from([(target.clone(), desired_value.clone())]));

        Ok(PlannedReplicaFieldPatch {
            target,
            desired_value,
            fields,
        })
    }
}

impl CommitStrategyExecutor for ReplicaConvergenceStrategy {
    fn execute(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &CanonicalStrategyCommitRequest,
        observation: &StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure> {
        let input = Self::parse_input(request)?;
        let existing_basis = observation
            .visibility()
            .entity_whole_aspects(execution, input.entity_id, [])?
            .ok_or_else(|| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::DomainRejection,
                    format!(
                        "replica convergence target entity {:?} is not visible in the committed basis",
                        input.entity_id
                    ),
                )
            })?;
        let replica_field_patch =
            Self::plan_replica_field_patch(observation, &existing_basis, input.desired_replicas)?;
        let existing = observation
            .visibility()
            .entity_whole_aspects(
                execution,
                input.entity_id,
                [replica_field_patch.target.aspect().aspect_key().clone()],
            )?
            .expect("entity was visible during strategy basis read");
        let desired_replicas_comparison_key =
            authoritative_aspect_value_field_comparison_key(&replica_field_patch.desired_value);

        let (action, mutation_program) = if existing
            .projected_field_comparison_key(&replica_field_patch.target)
            == Some(desired_replicas_comparison_key)
        {
            (
                ReplicaConvergenceAction::NoChange,
                StrategyMutationProgram::new(Vec::<WorkerIntentBatch>::new()),
            )
        } else {
            let batch =
                WorkerIntentBatch::new("replica-convergence-update").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: input.entity_id,
                        fields: replica_field_patch.fields,
                    }),
                ));
            (
                ReplicaConvergenceAction::UpdateReplicas,
                StrategyMutationProgram::new(vec![batch]),
            )
        };

        let output = Self::output_artifact(&ReplicaConvergenceOutput {
            entity_id: input.entity_id,
            action,
            desired_replicas: input.desired_replicas,
        });
        Ok(StrategyExecutionResult::new(output, mutation_program))
    }
}
