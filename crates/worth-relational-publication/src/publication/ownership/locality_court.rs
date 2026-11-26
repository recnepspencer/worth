//! The Native read-locality court builds ordinary intents and observes exact selected residency.
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::primitives::PartitionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::DeleteEntityIntent;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::UpdateEntityFieldsIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::origin_denial;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::transactions::data::primitives::CreatedEntityRef;
use contracts::transactions::data::primitives::EntitySpec;
use contracts::transactions::data::primitives::RecordRef;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

#[derive(Debug)]
pub struct CurrentOwnershipLocalityObservation {
    pub unrelated_roster_records: usize,
    pub owned_publications: usize,
    pub cold_before: usize,
    pub cold_after_unrelated: usize,
    pub cold_after_owned: usize,
    pub owned_refusal: contracts::publication::data::publication_error::PublicationError,
}
pub fn locality_error(detail: impl std::fmt::Display) -> TransactionCommitError {
    origin_denial(format!("current ownership locality court: {detail}"))
}

pub fn owner_kind(
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
) -> Result<KindId, TransactionCommitError> {
    basis
        .get_current_partition(owner.partition_id)
        .and_then(|p| p.entity_arena().get(&owner))
        .and_then(|r| r.kind_id())
        .ok_or_else(|| locality_error("real Query owner absent"))
}

pub fn seed_partitions(
    kind: KindId,
    seed_fields: impl Fn(u64) -> AspectFieldPatch,
) -> (WorkerIntentBatch, CreatedEntityRef) {
    let mut batch = WorkerIntentBatch::new("unrelated-native-partitions");
    let mut first = None;
    for part in 1..=64 {
        let partition = PartitionId(1000 + part);
        let key = contracts::symbols::data::client_key::ClientKey::raw(format!("unrelated-{part}"));
        let created = CreatedEntityRef {
            partition_id: partition,
            kind_id: kind,
            client_key: key.clone(),
        };
        if first.is_none() {
            first = Some(created);
        }
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: partition,
            kind_id: kind,
            client_key: key,
            fields: seed_fields(50_000 + u64::from(part)),
        })));
    }
    (batch, first.expect("the first seeded partition exists"))
}

pub fn field_edit(label: &str, entity_id: EntityId, fields: AspectFieldPatch) -> WorkerIntentBatch {
    WorkerIntentBatch::new(label).push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent { entity_id, fields },
    )))
}

pub fn inspect_recovered_locality(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
    unrelated: EntityId,
    value_fields: impl Fn(u64) -> AspectFieldPatch,
    mut check: impl FnMut(
        ExecutionRequest<'_, '_>,
        WorkerIntentBatch,
    ) -> Result<Result<(), TransactionCommitError>, TransactionCommitError>,
) -> Result<CurrentOwnershipLocalityObservation, TransactionCommitError> {
    use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess as _;
    let root = basis;
    let count = || {
        (1002..=1064)
            .filter(|p| {
                matches!(
                    root.probe_head_partition(PartitionId(*p)),
                    worth_relational_storage::internal_port::storage::residency::head_probe::PartitionHeadProbe::Cold
                )
            })
            .count()
    };
    let cold_before = count();
    // Keep the original host's request ceiling. This court guards physical
    // read locality; an additional fixed work invoice would conflate cold
    // index admission with the ordinary ownership selection being observed.
    execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |_| {
                    let bounded = *execution;
                    let selected =
                        super::select_current(&bounded, basis, &[RecordRef::Entity(unrelated)])?;
                    let unrelated_roster_records = selected.publications.len()
                        + selected.catalogs.len()
                        + selected.positions.len()
                        + selected.edges.len()
                        + selected.links.len();
                    check(
                        bounded,
                        field_edit("unrelated-value-edit", unrelated, value_fields(7)),
                    )??;
                    let cold_after_unrelated = count();
                    let selected =
                        super::select_current(&bounded, basis, &[RecordRef::Entity(owner)])?;
                    let owned_publications = selected.publications.len();
                    let member = selected
                        .origins()
                        .find(|o| o.meaning.source_scope == owner)
                        .and_then(|o| o.outputs.first())
                        .copied()
                        .ok_or_else(|| locality_error("real Query output absent"))?;
                    let refusal = check(
                        bounded,
                        WorkerIntentBatch::new("external-owned-edit").push(MutationIntent::Entity(
                            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: member }),
                        )),
                    )?
                    .err()
                    .ok_or_else(|| locality_error("external delete unexpectedly prepared"))?;
                    let TransactionCommitError::Publication {
                        error: owned_refusal,
                        ..
                    } = refusal
                    else {
                        return Err(refusal);
                    };
                    Ok(CurrentOwnershipLocalityObservation {
                        unrelated_roster_records,
                        owned_publications,
                        cold_before,
                        cold_after_unrelated,
                        cold_after_owned: count(),
                        owned_refusal,
                    })
                })
        })
        .map_err(|cause| TransactionCommitError::execution(cause.into()))?
        .map_err(|cause| TransactionCommitError::execution(cause.into()))?
        .0
}
