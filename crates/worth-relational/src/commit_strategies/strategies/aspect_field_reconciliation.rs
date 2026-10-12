use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey};

use crate::commit_strategies::data::{
    decode_aspect_field_locator, decode_aspect_value, decode_entity_id,
    encode_aspect_field_locator, encode_aspect_value, encode_entity_id,
    CanonicalStrategyCommitRequest, CanonicalStrategyOutputArtifact, CommitStrategyDescriptor,
    CommitStrategyExecutionRegistration, CommitStrategyExecutor, CommitStrategyFamilyName,
    CommitStrategyId, CommitStrategyRegistration, CommitStrategyRegistrationError,
    CommitStrategySemanticName, CommitStrategyVersion, NativeCodecError, NativeCodecReader,
    NativeStrategyCommitRequest, PersistentArtifactName, StrategyCallerProvenance,
    StrategyEntityAspectReadRecord, StrategyExecutionResult, StrategyExecutorFailure,
    StrategyExecutorFailureClass, StrategyExecutorFailureEvidence, StrategyInputSchemaName,
    StrategyInputSchemaVersion, StrategyIntentName, StrategyMutationProgram,
    StrategyObservationContext, StrategyOutputSchemaName, StrategyPacketContract,
    StrategyReadContract, StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
    StrategyTraversalBasis,
};
use crate::storage::data::authoritative_aspect_value_field_comparison_key;
use crate::transactions::data::AspectFieldPatch;
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectFieldReconciliationInput {
    pub entity_id: crate::identity::data::EntityId,
    pub field_locator: AspectFieldLocator,
    pub desired_value: AspectValue,
}

impl AspectFieldReconciliationInput {
    pub fn into_native_canonical_request(
        self,
        caller_provenance: StrategyCallerProvenance,
    ) -> Result<NativeStrategyCommitRequest, NativeCodecError> {
        let mut bytes = Vec::new();
        encode_entity_id(&mut bytes, self.entity_id);
        encode_aspect_field_locator(&mut bytes, &self.field_locator);
        encode_aspect_value(&mut bytes, &self.desired_value);
        Ok(NativeStrategyCommitRequest::from_native_canonical_bytes(
            CommitStrategySemanticName::new(
                AspectFieldReconciliationStrategy::DEFAULT_SEMANTIC_NAME,
            ),
            bytes,
            caller_provenance,
        ))
    }

    fn decode(bytes: &[u8]) -> Result<Self, NativeCodecError> {
        let mut reader = NativeCodecReader::new(bytes);
        let entity_id = decode_entity_id(&mut reader)?;
        let field_locator = decode_aspect_field_locator(&mut reader)?;
        let desired_value = decode_aspect_value(&mut reader)?;
        reader.finish()?;
        Ok(Self {
            entity_id,
            field_locator,
            desired_value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectFieldReconciliationOutput {
    pub entity_id: crate::identity::data::EntityId,
    pub field_locator: AspectFieldLocator,
    pub updated: bool,
}

impl AspectFieldReconciliationOutput {
    pub fn decode(bytes: &[u8]) -> Result<Self, NativeCodecError> {
        let mut reader = NativeCodecReader::new(bytes);
        let entity_id = decode_entity_id(&mut reader)?;
        let field_locator = decode_aspect_field_locator(&mut reader)?;
        let updated = reader.read_bool()?;
        reader.finish()?;
        Ok(Self {
            entity_id,
            field_locator,
            updated,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AspectFieldReconciliationStrategy;

impl AspectFieldReconciliationStrategy {
    pub const DEFAULT_SEMANTIC_NAME: &'static str = "strategy.aspect.field.reconcile";
    pub const DEFAULT_FAMILY_NAME: &'static str = "strategy.aspect";
    pub const DEFAULT_INPUT_SCHEMA_NAME: &'static str = "aspect.field.reconcile.input.v2";
    pub const DEFAULT_OUTPUT_SCHEMA_NAME: &'static str = "aspect.field.reconcile.output.v2";
    pub const DEFAULT_INTENT_NAME: &'static str = "aspect.scalar.field.reconcile";
    pub const DEFAULT_ARTIFACT_NAME: &'static str = "strategy.aspect.field.reconcile";

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
    ) -> Result<AspectFieldReconciliationInput, StrategyExecutorFailure> {
        AspectFieldReconciliationInput::decode(request.canonical_input().canonical_bytes()).map_err(
            |error| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::InvalidInput,
                    format!(
                        "aspect field reconciliation input could not be decoded: {}",
                        error.detail()
                    ),
                )
            },
        )
    }

    fn output_artifact(
        output: &AspectFieldReconciliationOutput,
    ) -> CanonicalStrategyOutputArtifact {
        let mut bytes = Vec::new();
        encode_entity_id(&mut bytes, output.entity_id);
        encode_aspect_field_locator(&mut bytes, &output.field_locator);
        bytes.push(u8::from(output.updated));
        CanonicalStrategyOutputArtifact::new(
            StrategyOutputSchemaName::new(Self::DEFAULT_OUTPUT_SCHEMA_NAME),
            bytes,
            PersistentArtifactName::new(Self::DEFAULT_ARTIFACT_NAME),
        )
    }

    fn require_declared_entity_scalar_field_locator(
        observation: &StrategyObservationContext<'_>,
        existing: &StrategyEntityAspectReadRecord,
        field_locator: &AspectFieldLocator,
    ) -> Result<FieldKey, StrategyExecutorFailure> {
        let lowered_plan = observation
            .entity_aspect_plan(existing.kind_id())
            .ok_or_else(|| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::DomainRejection,
                    format!(
                        "lowered foundational entity aspect plan is missing for kind {} during aspect reconciliation",
                        existing.kind_name()
                    ),
                )
            })?;
        let field = single_field_path(field_locator)?;
        let declared_aspect_key = lowered_plan
            .entity_scalar_field_aspect_key(&field)
            .ok_or_else(|| undeclared_locator_failure(existing, field_locator))?;
        if declared_aspect_key == field_locator.aspect().aspect_key().clone() {
            Ok(field)
        } else {
            Err(undeclared_locator_failure(existing, field_locator))
        }
    }
}

impl CommitStrategyExecutor for AspectFieldReconciliationStrategy {
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
                        "aspect field reconciliation target entity {:?} is not visible in the committed basis",
                        input.entity_id
                    ),
                )
            })?;
        let field_key = Self::require_declared_entity_scalar_field_locator(
            observation,
            &existing_basis,
            &input.field_locator,
        )?;
        let existing = observation
            .visibility()
            .entity_whole_aspects(
                execution,
                input.entity_id,
                [input.field_locator.aspect().aspect_key().clone()],
            )?
            .expect("entity was visible during strategy basis read");
        let desired_comparison_key =
            authoritative_aspect_value_field_comparison_key(&input.desired_value);
        let updated = existing.projected_field_comparison_key(&input.field_locator)
            != Some(desired_comparison_key);
        let mutation_program = if updated {
            let batch = WorkerIntentBatch::new("aspect-field-reconciliation-update").push(
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: input.entity_id,
                        fields: AspectFieldPatch::from_locator(
                            crate::transactions::data::planned_single_field_locator(
                                input.field_locator.aspect().aspect_key().clone(),
                                field_key,
                            ),
                            input.desired_value.clone(),
                        ),
                    },
                )),
            );
            StrategyMutationProgram::new(vec![batch])
        } else {
            StrategyMutationProgram::new(Vec::<WorkerIntentBatch>::new())
        };

        let output = Self::output_artifact(&AspectFieldReconciliationOutput {
            entity_id: input.entity_id,
            field_locator: input.field_locator,
            updated,
        });
        Ok(StrategyExecutionResult::new(output, mutation_program))
    }
}

fn single_field_path(
    field_locator: &AspectFieldLocator,
) -> Result<FieldKey, StrategyExecutorFailure> {
    match field_locator.field_path().fields() {
        [field] => Ok(field.clone()),
        _ => Err(StrategyExecutorFailure::with_evidence(
            StrategyExecutorFailureClass::InvalidInput,
            "aspect field reconciliation target must be a single foundational field path",
            StrategyExecutorFailureEvidence::AspectFieldLocator {
                locator: field_locator.clone(),
            },
        )),
    }
}

fn undeclared_locator_failure(
    existing: &StrategyEntityAspectReadRecord,
    field_locator: &AspectFieldLocator,
) -> StrategyExecutorFailure {
    StrategyExecutorFailure::with_evidence(
        StrategyExecutorFailureClass::DomainRejection,
        format!(
            "aspect field locator is not a lowered foundational scalar entity aspect on kind {}",
            existing.kind_name()
        ),
        StrategyExecutorFailureEvidence::AspectFieldLocator {
            locator: field_locator.clone(),
        },
    )
}

#[cfg(test)]
mod tests;
