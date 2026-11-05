//! Compare independently written current facts once at region readmission/load.
use super::*;
use crate::{
    publication::ownership::{checkpoint, record, request},
    schema::data::RelationalSchemaRegistry,
    storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    storage::overlay::PartitionState,
    storage::residency::RelationalResidencyDemandFailure as Failure,
    transactions::data::*,
};
use std::collections::HashMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(crate) fn validate(
    execution: &ExecutionRequest<'_, '_>,
    partition: &PartitionState,
    registry: &RelationalSchemaRegistry,
) -> Result<(), Failure> {
    let Some(storage) = &registry.current_publication_storage else {
        return Ok(());
    };
    let mut result = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        result = Some((|| {
            let mut funding = execution
                .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
                .map_err(request)?
                .map_err(|e| request(crate::execution::denial::lease_denial(e)))?;
            let mut positions = HashMap::new();
            let mut publications = HashMap::new();
            let mut catalogs = Vec::new();
            for slot in partition.entity_arena.live_bitset.set_slots() {
                checkpoint(context, 1)?;
                let row = partition
                    .entity_arena
                    .get_slot(slot)
                    .ok_or_else(|| damaged(partition))?;
                let Some(kind) = row.kind_id() else {
                    continue;
                };
                if kind != storage.position_kind
                    && kind != storage.publication_kind
                    && kind != storage.catalog_kind
                {
                    continue;
                }
                let id = EntityId::new(partition.partition_id, slot as u64, row.generation());
                let aspect = if kind == storage.position_kind {
                    crate::schema::current_publication_ownership::POSITION
                } else if kind == storage.publication_kind {
                    crate::schema::current_publication_ownership::PUBLICATION
                } else {
                    crate::schema::current_publication_ownership::CATALOG
                };
                let (publication, bytes) = partition
                    .entity_arena
                    .metadata_history_at(slot)
                    .and_then(|history| history.len().checked_sub(1).and_then(|i| history.get(i)))
                    .and_then(|m| m.authoritative_aspect_state.as_ref())
                    .and_then(|s| record::fields(s, aspect))
                    .ok_or_else(|| damaged(partition))?;
                let bad = |source| corrupt(publication, source, id);
                if kind == storage.catalog_kind {
                    let value: record::Catalog = crate::publication::ownership::decode_payload(
                        bytes,
                        publication,
                        &RecordRef::Entity(id),
                        &mut funding,
                        context,
                    )?;
                    if publication != CommitId(0) {
                        return Err(damaged(partition));
                    }
                    catalogs.push((id, value));
                } else if kind == storage.position_kind {
                    let value: RoutePresence = crate::publication::ownership::decode_payload(
                        bytes,
                        publication,
                        &RecordRef::Entity(id),
                        &mut funding,
                        context,
                    )?;
                    let source = value.meaning.source_scope;
                    let key = (
                        value.meaning.family.clone(),
                        source,
                        value.meaning.source_partition,
                    );
                    if value.publication != publication
                        || !value.valid_digest()
                        || positions.insert(key, value).is_some()
                    {
                        return Err(bad(source));
                    }
                } else {
                    let value: record::Publication = crate::publication::ownership::decode_payload(
                        bytes,
                        publication,
                        &RecordRef::Entity(id),
                        &mut funding,
                        context,
                    )?;
                    let source = value.origin.meaning.source_scope;
                    let key = (
                        value.origin.meaning.family.clone(),
                        source,
                        value.origin.meaning.source_partition,
                    );
                    use sha2::Digest;
                    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
                    if value.origin.commit.commit_id != publication
                        || publications.insert(key, (id, value, digest)).is_some()
                    {
                        return Err(bad(source));
                    }
                }
            }
            let mut routes = HashMap::new();
            for (id, catalog) in &catalogs {
                for route in &catalog.routes {
                    checkpoint(context, 1)?;
                    let key = (
                        &route.meaning.family,
                        route.meaning.source_scope,
                        route.meaning.source_partition,
                    );
                    if route.meaning.source_scope != catalog.scope
                        || routes.insert(key, route).is_some()
                    {
                        return Err(corrupt(route.publication, catalog.scope, *id));
                    }
                }
            }
            for (key, (id, row, digest)) in publications {
                checkpoint(context, 1)?;
                let o = &row.origin;
                let bad = || corrupt(o.commit.commit_id, o.meaning.source_scope, id);
                let p = positions.remove(&key).ok_or_else(bad)?;
                let route = *routes.get(&(&key.0, key.1, key.2)).ok_or_else(bad)?;
                routes.remove(&(
                    &route.meaning.family,
                    route.meaning.source_scope,
                    route.meaning.source_partition,
                ));
                if !record::route_matches(&row, &digest, route) {
                    return Err(bad());
                }
                if p.meaning != o.meaning
                    || p.publication != o.commit.commit_id
                    || p.generation != o.commit.version_id
                    || p.row_digest != digest
                {
                    return Err(bad());
                }
            }
            if let Some(route) = routes
                .into_values()
                .min_by_key(|r| (r.publication, r.meaning.source_scope))
            {
                return Err(corrupt(
                    route.publication,
                    route.meaning.source_scope,
                    route.meaning.source_scope,
                ));
            }
            if let Some(p) = positions
                .into_values()
                .min_by_key(|p| (p.publication, p.meaning.source_scope))
            {
                return Err(corrupt(
                    p.publication,
                    p.meaning.source_scope,
                    p.meaning.source_scope,
                ));
            }
            Ok(())
        })());
    });
    let result = match (handoff, result) {
        (Ok(_), Some(result)) => result,
        (Err(_), Some(Err(error))) => Err(error),
        (Err(stop), _) => Err(request(stop)),
        (Ok(_), None) => unreachable!("completed current-fact admission ran"),
    };
    result.map_err(|e| failure(e, partition.partition_id))
}
fn corrupt(publication: CommitId, source: EntityId, record: EntityId) -> TransactionCommitError {
    let mut error = crate::publication::data::PublicationError::authored_retirement(
        crate::publication::data::AuthoredRetirementRefusal::CorruptCurrentOwnership {
            publication,
        },
        RecordRef::Entity(source),
    );
    if source != record {
        error
            .context
            .affected_records
            .push(RecordRef::Entity(record));
    }
    TransactionCommitError::publication(error)
}
fn damaged(partition: &PartitionState) -> TransactionCommitError {
    request(
        crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
            RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition.partition_id),
                cause: Cause::Damaged,
            },
        ),
    )
}
fn failure(
    error: TransactionCommitError,
    partition: crate::identity::data::PartitionId,
) -> Failure {
    match error {
        TransactionCommitError::Execution { denial, .. } => match denial.kind {
            CommitExecutionDenialKind::Cause(cause) => Failure::Request(cause),
        },
        TransactionCommitError::Publication { error, .. } => {
            let publication = match error.authored_retirement_refusal() {
                Some(
                    crate::publication::data::AuthoredRetirementRefusal::CorruptCurrentOwnership {
                        publication,
                    },
                ) => publication,
                _ => unreachable!("closed current-fact grammar returns custody corruption"),
            };
            let mut records = error.context.affected_records.iter().filter_map(|r| {
                if let RecordRef::Entity(id) = r {
                    Some(*id)
                } else {
                    None
                }
            });
            let source = records
                .next()
                .expect("current-fact corruption names its subject");
            let record = records.next().unwrap_or(source);
            RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition),
                cause: Cause::CorruptCurrentOwnership {
                    publication,
                    source,
                    record,
                },
            }
            .into()
        }
        _ => unreachable!("current-fact admission performs no mutation or conflict arbitration"),
    }
}
