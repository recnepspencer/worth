//! Continuity of a named record through public persistent-name resolution.
//!
//! A record born under a persistent name answers continuity only from
//! Relational's branch-local derived name index, exactly as the retained
//! observation sees it: its name is resolved over all live descendants at
//! the observation's commit and the answer is projected onto the Bridge
//! topology. A record with no persistent name has no name to resolve and
//! keeps the indexed record-history walk.

use worth_relational::facade::change_source::RelationalObservationReadDenial;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{
    LineageRecordTarget, PersistentNameOfRecord, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameResolutionOutcome, PersistentNameSelection,
    PersistentNameTarget, RelationalPersistentNameReadDenial,
};
use worth_relational::facade::mvcc::RelationalBranchObservation;
use worth_relational::facade::transactions::RecordRef;

use super::RuntimeBridgeRelationalSource;
use crate::facade::{
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeLineageSourceError, BridgeLineageSourceErrorKind,
};
use crate::relational_source::identities::record_ref_identity;

/// A named record's resolution at the retained observation.
pub(super) struct NamedContinuity {
    outcome: PersistentNameResolutionOutcome,
}

/// Resolves the record's persistent name over all live descendants, or
/// `None` when the record carries no persistent name.
///
/// Complexity: O(1) admission plus O(lineage path from the record's name).
pub(super) fn resolve_named_continuity(
    source: &RuntimeBridgeRelationalSource,
    observation: &RelationalBranchObservation,
    entity_id: EntityId,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<NamedContinuity>, BridgeLineageSourceError> {
    source.runtime.with_runtime(|runtime| {
        runtime
            .with_persistent_names_at_observation(execution, observation, |view| {
                let (PersistentNameOfRecord::Live { name, .. }
                | PersistentNameOfRecord::Ended { name, .. }) =
                    view.name_of_record(LineageRecordTarget::Entity(entity_id))
                else {
                    return None;
                };
                let outcome = view.resolve(name, &PersistentNameSelection::AllDescendants);
                Some(NamedContinuity { outcome })
            })
            .map_err(name_read_denial)
    })
}

/// Projects a named resolution onto the Bridge lineage authority.
///
/// The live descendants are the successor records and their lineages the
/// resolved lineages, exactly as the record-history projection reports the
/// lineages current at the observation: one live descendant (the name still
/// holding a regenerated or restored record, a one-child split, a split whose
/// siblings retired, or a merge's keeper) is a single successor; two or more
/// are split successors; none is no authoritative successor. Ended lineages
/// on the walked path are never reported as resolved. Name resolution walks
/// names, not events, so it reports no traversed event ids.
pub(super) fn project_named_authority(
    source: &RuntimeBridgeRelationalSource,
    request: BridgeHistoricalLineageRequest,
    named: NamedContinuity,
) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
    let live = match named.outcome.result {
        Ok(PersistentNameResolution::Exact(target)) => vec![target],
        Ok(PersistentNameResolution::Plural(targets)) => targets,
        Ok(PersistentNameResolution::Retired { .. }) => Vec::new(),
        Err(denial) => return Err(resolution_denial(&denial)),
    };
    let mut lineage_identities: Vec<_> = live
        .iter()
        .filter_map(PersistentNameTarget::lineage)
        .map(|lineage| {
            BridgeHistoricalResolvedLineageIdentity::from_relational_lineage_id(lineage.0)
        })
        .collect();
    lineage_identities.sort_unstable();
    lineage_identities.dedup();
    let mut record_identities: Vec<_> = live
        .iter()
        .filter_map(|target| match target {
            PersistentNameTarget::Entity { entity, .. } => Some(*entity),
            _ => None,
        })
        .filter(|entity| source.admits_relational_partition(entity.partition_id.as_u32()))
        .map(|entity| {
            BridgeHistoricalResolvedRecordIdentity::from_relational_record(record_ref_identity(
                &RecordRef::Entity(entity),
            ))
        })
        .collect();
    record_identities.sort_unstable();
    record_identities.dedup();
    BridgeHistoricalLineageAuthority::try_new(
        request.authority_basis().clone(),
        lineage_identities,
        record_identities,
        Vec::new(),
    )
}

fn name_read_denial(denial: RelationalPersistentNameReadDenial) -> BridgeLineageSourceError {
    match denial {
        RelationalPersistentNameReadDenial::Observation(read) => observation_denial(read),
        other => BridgeLineageSourceError::new(
            BridgeLineageSourceErrorKind::HistoricalResolutionFailure,
            format!("bridge continuity persistent-name read refused: {other:?}"),
        ),
    }
}

fn observation_denial(denial: RelationalObservationReadDenial) -> BridgeLineageSourceError {
    BridgeLineageSourceError::new(
        BridgeLineageSourceErrorKind::ObservationRead(denial),
        format!("bridge continuity lineage read refused: {denial:?}"),
    )
}

fn resolution_denial(denial: &PersistentNameResolutionDenial) -> BridgeLineageSourceError {
    BridgeLineageSourceError::new(
        BridgeLineageSourceErrorKind::HistoricalResolutionFailure,
        format!("bridge continuity persistent-name resolution refused: {denial:?}"),
    )
}
