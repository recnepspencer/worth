use serde::{Deserialize, Serialize};

use worth_foundational::facade::FieldKey;

use crate::commit_strategies::data::{
    decode_aspect_field_patch, decode_entity_id, decode_string, encode_aspect_field_patch,
    encode_entity_id, encode_string, CanonicalStrategyCommitRequest,
    CanonicalStrategyOutputArtifact, CommitStrategyDescriptor, CommitStrategyExecutionRegistration,
    CommitStrategyExecutor, CommitStrategyFamilyName, CommitStrategyId, CommitStrategyRegistration,
    CommitStrategyRegistrationError, CommitStrategySemanticName, CommitStrategyVersion,
    NativeCodecError, NativeCodecReader, NativeStrategyCommitRequest, PersistentArtifactName,
    StrategyCallerProvenance, StrategyEntityAspectReadRecord, StrategyExecutionResult,
    StrategyExecutorFailure, StrategyExecutorFailureClass, StrategyExecutorFailureEvidence,
    StrategyInputSchemaName, StrategyInputSchemaVersion, StrategyIntentName,
    StrategyMutationProgram, StrategyObservationContext, StrategyOutputSchemaName,
    StrategyPacketContract, StrategyReadContract, StrategyReadCostClass, StrategyReadLocalityClass,
    StrategyReadScopeClass, StrategyTraversalBasis,
};
use crate::identity::data::EntityId;
use crate::storage::data::authoritative_aspect_value_field_comparison_key;
use crate::symbols::data::ClientKey;
use crate::transactions::data::AspectFieldPatch;
use crate::transactions::data::{
    EntityMutationIntent, EntitySpec, MutationIntent, ReplaceEntityIntent, WorkerIntentBatch,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityReplacementReconciliationInput {
    pub entity_id: EntityId,
    pub replacement_client_key: String,
    pub desired_aspect_fields: AspectFieldPatch,
}

impl EntityReplacementReconciliationInput {
    pub fn into_native_canonical_request(
        self,
        caller_provenance: StrategyCallerProvenance,
    ) -> Result<NativeStrategyCommitRequest, NativeCodecError> {
        let mut bytes = Vec::new();
        encode_entity_id(&mut bytes, self.entity_id);
        encode_string(&mut bytes, &self.replacement_client_key);
        encode_aspect_field_patch(&mut bytes, &self.desired_aspect_fields)?;
        Ok(NativeStrategyCommitRequest::from_native_canonical_bytes(
            CommitStrategySemanticName::new(
                EntityReplacementReconciliationStrategy::DEFAULT_SEMANTIC_NAME,
            ),
            bytes,
            caller_provenance,
        ))
    }

    fn decode(bytes: &[u8]) -> Result<Self, NativeCodecError> {
        let mut reader = NativeCodecReader::new(bytes);
        let entity_id = decode_entity_id(&mut reader)?;
        let replacement_client_key = decode_string(&mut reader)?;
        let desired_aspect_fields = decode_aspect_field_patch(&mut reader)?;
        reader.finish()?;
        Ok(Self {
            entity_id,
            replacement_client_key,
            desired_aspect_fields,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityReplacementReconciliationAction {
    NoChange,
    ReplaceEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityReplacementReconciliationOutput {
    pub entity_id: EntityId,
    pub action: EntityReplacementReconciliationAction,
    pub replacement_client_key: String,
}

impl EntityReplacementReconciliationOutput {
    pub fn decode(bytes: &[u8]) -> Result<Self, NativeCodecError> {
        let mut reader = NativeCodecReader::new(bytes);
        let entity_id = decode_entity_id(&mut reader)?;
        let action = match reader.read_u8()? {
            0 => EntityReplacementReconciliationAction::NoChange,
            1 => EntityReplacementReconciliationAction::ReplaceEntity,
            tag => {
                return Err(NativeCodecError::new(format!(
                    "unknown entity replacement action tag {tag}"
                )))
            }
        };
        let replacement_client_key = decode_string(&mut reader)?;
        reader.finish()?;
        Ok(Self {
            entity_id,
            action,
            replacement_client_key,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EntityReplacementReconciliationStrategy;

impl EntityReplacementReconciliationStrategy {
    pub const DEFAULT_SEMANTIC_NAME: &'static str = "strategy.replace.entity.reconcile";
    pub const DEFAULT_FAMILY_NAME: &'static str = "strategy.replace";
    pub const DEFAULT_INPUT_SCHEMA_NAME: &'static str = "entity.replace.reconcile.input.v1";
    pub const DEFAULT_OUTPUT_SCHEMA_NAME: &'static str = "entity.replace.reconcile.output.v1";
    pub const DEFAULT_INTENT_NAME: &'static str = "entity.replace.reconcile";
    pub const DEFAULT_ARTIFACT_NAME: &'static str = "strategy.replace.entity.reconcile";

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
    ) -> Result<EntityReplacementReconciliationInput, StrategyExecutorFailure> {
        EntityReplacementReconciliationInput::decode(request.canonical_input().canonical_bytes())
            .map_err(|error| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::InvalidInput,
                    format!(
                        "entity replacement reconciliation input could not be decoded: {}",
                        error.detail()
                    ),
                )
            })
    }

    fn output_artifact(
        output: &EntityReplacementReconciliationOutput,
    ) -> CanonicalStrategyOutputArtifact {
        let mut bytes = Vec::new();
        encode_entity_id(&mut bytes, output.entity_id);
        bytes.push(match output.action {
            EntityReplacementReconciliationAction::NoChange => 0,
            EntityReplacementReconciliationAction::ReplaceEntity => 1,
        });
        encode_string(&mut bytes, &output.replacement_client_key);
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
        let lowered_plan =
            observation
                .entity_aspect_plan(existing.kind_id())
                .ok_or_else(|| {
                    StrategyExecutorFailure::new(
                        StrategyExecutorFailureClass::DomainRejection,
                        format!(
                            "lowered foundational entity aspect plan is missing for kind {} during replacement reconciliation",
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

    fn reconcile_fields(
        observation: &StrategyObservationContext<'_>,
        existing: &StrategyEntityAspectReadRecord,
        desired_aspect_fields: &AspectFieldPatch,
    ) -> Result<AspectFieldPatch, StrategyExecutorFailure> {
        for target in desired_aspect_fields.locators() {
            let [field] = target.field_path().fields() else {
                return Err(StrategyExecutorFailure::with_evidence(
                    StrategyExecutorFailureClass::DomainRejection,
                    "entity replacement reconciliation target field path is not a single foundational field path",
                    StrategyExecutorFailureEvidence::AspectFieldLocator {
                        locator: target.clone(),
                    },
                ));
            };
            let aspect_key =
                Self::require_lowered_entity_scalar_field(observation, existing, field)?;
            if &aspect_key != target.aspect().aspect_key() {
                return Err(StrategyExecutorFailure::with_evidence(
                    StrategyExecutorFailureClass::DomainRejection,
                    "entity replacement reconciliation target does not match lowered scalar aspect",
                    StrategyExecutorFailureEvidence::AspectFieldLocatorMismatch {
                        locator: target.clone(),
                        expected_aspect_key: aspect_key,
                    },
                ));
            }
        }
        Ok(desired_aspect_fields.clone())
    }

    fn authoritative_fields_match(
        existing: &StrategyEntityAspectReadRecord,
        desired_aspect_fields: &AspectFieldPatch,
    ) -> bool {
        desired_aspect_fields.iter().all(|(target, value)| {
            let [_field] = target.field_path().fields() else {
                return false;
            };
            let desired_comparison_key = authoritative_aspect_value_field_comparison_key(value);
            existing.projected_field_comparison_key(target) == Some(desired_comparison_key)
        })
    }
}

impl CommitStrategyExecutor for EntityReplacementReconciliationStrategy {
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
                        "entity replacement reconciliation target entity {:?} is not visible in the committed basis",
                        input.entity_id
                    ),
                )
            })?;
        let desired_aspect_fields =
            Self::reconcile_fields(observation, &existing_basis, &input.desired_aspect_fields)?;
        let projected_aspect_keys = desired_aspect_fields
            .locators()
            .map(|locator| locator.aspect().aspect_key().clone());
        let existing = observation
            .visibility()
            .entity_whole_aspects(execution, input.entity_id, projected_aspect_keys)?
            .expect("entity was visible during strategy basis read");
        let unchanged = input.replacement_client_key.is_empty()
            && Self::authoritative_fields_match(&existing, &desired_aspect_fields);

        let (action, mutation_program) = if unchanged {
            (
                EntityReplacementReconciliationAction::NoChange,
                StrategyMutationProgram::new(Vec::<WorkerIntentBatch>::new()),
            )
        } else {
            let batch = WorkerIntentBatch::new("entity-replacement-reconciliation").push(
                MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: input.entity_id,
                    replacement: EntitySpec {
                        partition_id: input.entity_id.partition_id,
                        kind_id: existing.kind_id(),
                        client_key: ClientKey::raw(input.replacement_client_key.clone()),
                        fields: desired_aspect_fields,
                    },
                })),
            );
            (
                EntityReplacementReconciliationAction::ReplaceEntity,
                StrategyMutationProgram::new(vec![batch]),
            )
        };

        let output = Self::output_artifact(&EntityReplacementReconciliationOutput {
            entity_id: input.entity_id,
            action,
            replacement_client_key: input.replacement_client_key,
        });
        Ok(StrategyExecutionResult::new(output, mutation_program))
    }
}

#[cfg(test)]
mod tests;
