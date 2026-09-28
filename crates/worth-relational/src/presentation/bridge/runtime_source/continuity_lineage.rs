use worth_runtime_bridge::facade::{
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeLineageSourceError, BridgeLineageSourceErrorKind, ContinuityLineageSource,
};

use super::{
    observation_bindings::RelationalBridgeSelectedObservation, RuntimeBridgeRelationalSource,
};
use crate::facade::identity::EntityId;
use crate::facade::lineage::HistoricalLineageResolution;
use crate::facade::transactions::RecordRef;
use crate::presentation::bridge::identities::{
    record_ref_from_identity_parts, record_ref_identity,
};

impl ContinuityLineageSource for RuntimeBridgeRelationalSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        let admitted = admit_lineage_request(self, request)?;
        let resolved = resolve_exact_lineage(self, admitted)?;
        project_lineage_authority(self, resolved)
    }
}

struct AdmittedLineageRequest {
    request: BridgeHistoricalLineageRequest,
    observation: RelationalBridgeSelectedObservation,
    entity_id: EntityId,
}

struct ResolvedLineageRequest {
    request: BridgeHistoricalLineageRequest,
    resolution: HistoricalLineageResolution,
    visible_entities: Vec<EntityId>,
}

fn admit_lineage_request(
    source: &RuntimeBridgeRelationalSource,
    request: BridgeHistoricalLineageRequest,
) -> Result<AdmittedLineageRequest, BridgeLineageSourceError> {
    let observation = source
        .observation_bindings
        .resolve(request.authority_basis().snapshot_identity())
        .map_err(|error| lineage_error(error.to_string()))?;
    let observed_branch =
        worth_runtime_bridge::facade::TruthBranchIdentity::from_relational_branch_id(
            observation.branch_id().0.clone(),
        );
    if request.authority_basis().branch_identity() != &observed_branch {
        return Err(lineage_error(
            "bridge continuity branch does not belong to the retained observation",
        ));
    }
    let identity = request
        .prior_slice()
        .relational_record_identity_parts()
        .ok_or_else(|| {
            lineage_error("relational bridge continuity requires typed record identity parts")
        })?;
    if !source.admits_relational_partition(identity.partition_id()) {
        return Err(lineage_error(
            "relational bridge lineage request is outside the source partition authority",
        ));
    }
    let record = record_ref_from_identity_parts(identity)
        .map_err(|error| lineage_error(error.to_string()))?;
    let RecordRef::Entity(entity_id) = record else {
        return Err(BridgeLineageSourceError::new(
            BridgeLineageSourceErrorKind::UnsupportedContinuityClass,
            "bridge continuity lineage adapter currently supports entity record identities only",
        ));
    };
    Ok(AdmittedLineageRequest {
        request,
        observation,
        entity_id,
    })
}

fn resolve_exact_lineage(
    source: &RuntimeBridgeRelationalSource,
    admitted: AdmittedLineageRequest,
) -> Result<ResolvedLineageRequest, BridgeLineageSourceError> {
    let entity_label = format!(
        "entity:{}:{}:{}",
        admitted.entity_id.partition_id.0,
        admitted.entity_id.local_slot.0,
        admitted.entity_id.generation.0
    );
    let (resolution, visible_entities) = source.runtime.with_runtime(|runtime| {
        let resolution = runtime
            .record_history_at_observation(admitted.observation.observation(), admitted.entity_id)
            .ok_or_else(|| {
                lineage_error(format!(
                    "bridge continuity lineage adapter could not resolve record history for `{entity_label}`"
                ))
            })?;
        let visible_entities = runtime.visible_entities_for_lineages_at_observation(
            admitted.observation.observation(),
            &resolution.resolved,
        );
        Ok((resolution, visible_entities))
    })?;
    Ok(ResolvedLineageRequest {
        request: admitted.request,
        resolution,
        visible_entities,
    })
}

fn project_lineage_authority(
    source: &RuntimeBridgeRelationalSource,
    resolved: ResolvedLineageRequest,
) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
    let mut lineage_identities = resolved
        .resolution
        .resolved
        .iter()
        .map(|lineage| {
            BridgeHistoricalResolvedLineageIdentity::from_relational_lineage_id(lineage.0)
        })
        .collect::<Vec<_>>();
    canonicalize(&mut lineage_identities);

    let mut record_identities = resolved
        .visible_entities
        .into_iter()
        .filter(|entity| source.admits_relational_partition(entity.partition_id.as_u32()))
        .map(|entity| {
            BridgeHistoricalResolvedRecordIdentity::from_relational_record(record_ref_identity(
                &RecordRef::Entity(entity),
            ))
        })
        .collect::<Vec<_>>();
    canonicalize(&mut record_identities);

    let mut event_ids = resolved.resolution.traversed_event_ids;
    canonicalize(&mut event_ids);
    BridgeHistoricalLineageAuthority::try_new(
        resolved.request.authority_basis().clone(),
        lineage_identities,
        record_identities,
        event_ids,
    )
}

fn canonicalize<T: Ord>(items: &mut Vec<T>) {
    items.sort_unstable();
    items.dedup();
}

fn lineage_error(detail: impl Into<std::sync::Arc<str>>) -> BridgeLineageSourceError {
    BridgeLineageSourceError::new(
        BridgeLineageSourceErrorKind::HistoricalResolutionFailure,
        detail,
    )
}
