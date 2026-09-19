use std::sync::Arc;

use super::mutation_program::{
    compute_mutation_program_digest, forged_mutation_program_for_digest_test,
};
use super::output_artifact::{compute_output_digest, forged_output_artifact_for_digest_test};
use super::{
    CanonicalStrategyOutputArtifact, StrategyMutationProgram, StrategyMutationProgramDigest,
};
use crate::commit_strategies::data::{PersistentArtifactName, StrategyOutputSchemaName};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    AspectFieldPatch, DeleteEntityIntent, EntityMutationIntent, EntitySpec, RevalidateEntityIntent,
    UpdateEntityFieldsIntent,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};

fn artifact() -> CanonicalStrategyOutputArtifact {
    CanonicalStrategyOutputArtifact::new(
        StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
        b"status=ok".to_vec(),
        PersistentArtifactName::new("strategy.intent.reconcile"),
    )
}

fn mutation_program() -> StrategyMutationProgram {
    StrategyMutationProgram::new(vec![WorkerIntentBatch::new("reconcile").push(
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId(1),
            kind_id: KindId(1),
            client_key: ClientKey::from("deployment-a"),
            fields: AspectFieldPatch::from_locator(
                crate::transactions::data::planned_single_field_locator(
                    AspectKey::new("name").expect("valid name aspect key"),
                    FieldKey::new("name").expect("valid name field key"),
                ),
                AspectValue::String(InternedString::Raw("deployment-a".to_string())),
            ),
        })),
    )])
}

#[test]
fn output_artifact_constructor_preserves_verified_digest() {
    let artifact = artifact();

    assert_eq!(
        artifact.digest(),
        compute_output_digest(artifact.canonical_bytes())
    );
    assert_eq!(artifact.canonical_bytes(), b"status=ok");
}

#[test]
fn output_artifact_digest_drift_is_detectable_with_typed_fixture() {
    let artifact = artifact();
    let forged =
        forged_output_artifact_for_digest_test(&artifact, Arc::from(b"bad=true".as_slice()));

    assert_ne!(
        forged.digest(),
        compute_output_digest(forged.canonical_bytes())
    );
}

#[test]
fn mutation_program_constructor_preserves_verified_digest_and_intent_count() {
    let program = mutation_program();

    assert_eq!(
        program.digest(),
        compute_mutation_program_digest(program.worker_batches())
    );
    assert_eq!(program.total_intent_count(), 1);
}

#[test]
fn mutation_program_digest_drift_is_detectable_with_typed_fixture() {
    let program = mutation_program();
    let forged =
        forged_mutation_program_for_digest_test(&program, StrategyMutationProgramDigest([0; 32]));

    assert_ne!(
        forged.digest(),
        compute_mutation_program_digest(forged.worker_batches())
    );
}

/// The one record every intent in the discrimination court below names.
fn shared_record() -> EntityId {
    EntityId::new(PartitionId(1), 7, 1)
}

fn program_over_shared_record(
    label: &str,
    intent: EntityMutationIntent,
) -> StrategyMutationProgram {
    StrategyMutationProgram::new(vec![
        WorkerIntentBatch::new(label).push(MutationIntent::Entity(intent))
    ])
}

#[test]
fn the_program_digest_separates_opposite_intents_over_one_record() {
    let record = shared_record();
    let revalidate = program_over_shared_record(
        "revalidate",
        EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id: record }),
    );
    let delete = program_over_shared_record(
        "delete",
        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: record }),
    );
    let update = program_over_shared_record(
        "update",
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: record,
            fields: AspectFieldPatch::default(),
        }),
    );

    // A demand, a deletion and a rewrite of one record carry the same entity id
    // and nothing else, so the term tag is the only thing in the canonical bytes
    // that tells them apart. Two commits of opposite meaning sharing a program
    // digest would make replay reproduce the wrong one.
    assert_ne!(
        revalidate.digest(),
        delete.digest(),
        "a demand leaves the record alive and a deletion removes it; one digest cannot stand for both"
    );
    assert_ne!(
        revalidate.digest(),
        update.digest(),
        "a demand authors nothing and a rewrite authors fields; one digest cannot stand for both"
    );
    assert_ne!(
        delete.digest(),
        update.digest(),
        "a deletion and a rewrite of one record must remain distinguishable"
    );
    assert_eq!(
        revalidate.digest(),
        compute_mutation_program_digest(revalidate.worker_batches()),
        "the demand's program must carry the digest its own canonical bytes produce"
    );
}
