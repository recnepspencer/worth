//! Public Native publication succession and authoritative payload observations.
mod partial_fields;
mod retirement;
mod routes;
use worth_execution::ExecutionRequest;
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::{
        RelationalInstalledApplicationPublicationAuthority, RelationalPublicationOutcome,
        RelationalRequiredOutputPublication, RequiredOutputProducingImplementation,
        RequiredOutputPublication, RequiredOutputPublicationMeaning, RequiredOutputRolePosture,
        RequiredOutputRoleTarget,
    },
    runtime::RelationalRuntime,
    transactions::{
        CommitResult, CreatedEntityRef, EntityReference, TransactionCommitError, WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::publications::begin_main_transaction;

fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)?
        .bind_required_output_publication(installed, publication)?;
    let candidate = runtime.prepare_validated_proposal(execution, proposal)?;
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("native completion must perform")
    };
    runtime.settle_performed_publication(performed)
}

fn preserve(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    previous: RelationalRequiredOutputPublication,
    ids: [EntityId; 2],
    dependency: u8,
) -> RequiredOutputPublication {
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("native-preserver", [dependency; 32]),
            previous.authority(),
        )
        .unwrap();
    let meaning = previous.meaning().clone();
    RequiredOutputPublication::Preserve {
        admission,
        predecessor: previous,
        meaning,
        roles: ids
            .into_iter()
            .enumerate()
            .map(|(index, id)| RequiredOutputRoleTarget {
                role: format!("preserved-{index}"),
                entity_name: "native-unit-output".into(),
                kind: KindId(1),
                posture: RequiredOutputRolePosture::Preserve,
                target: EntityReference::Existing(id),
            })
            .collect(),
    }
}

fn native_birth_fixture(
    admission: worth_proof::AdmittedRequiredOutputPublication,
    targets: Vec<CreatedEntityRef>,
) -> RequiredOutputPublication {
    let meaning = native_meaning_fixture(admission.producer());
    RequiredOutputPublication::Birth {
        admission,
        absence: None,
        meaning,
        roles: targets
            .into_iter()
            .enumerate()
            .map(|(index, target)| RequiredOutputRoleTarget {
                role: format!("output-{index}"),
                entity_name: "native-unit-output".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(target),
            })
            .collect(),
    }
}

fn native_roles_fixture(
    roles: &[(EntityId, KindId)],
    posture: RequiredOutputRolePosture,
) -> Vec<RequiredOutputRoleTarget> {
    roles
        .iter()
        .enumerate()
        .map(|(index, (id, kind))| RequiredOutputRoleTarget {
            role: match posture {
                RequiredOutputRolePosture::Create => format!("output-{index}"),
                _ => format!("preserved-{index}"),
            },
            entity_name: "native-unit-output".into(),
            kind: *kind,
            posture,
            target: EntityReference::Existing(*id),
        })
        .collect()
}

fn native_meaning_fixture(family: &str) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: family.to_owned(),
        source_scope: EntityId::new(PartitionId::main(), 0, 1),
        source_partition: CanonicalDigestId::new([0; 32]),
        source_epoch: CanonicalDigestId::new([0; 32]),
    }
}
