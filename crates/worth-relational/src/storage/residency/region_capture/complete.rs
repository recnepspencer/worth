//! Original Full-only REG6 member emission without owned checkpoint images.
mod adjacency;
pub(in crate::storage::residency) mod arena;
mod aspects;
mod entity;
mod relation;
use super::super::region_image::{RelationalRegionImage, RelationalRegionImageKind};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalHistoryMemberDirectory,
    RelationalRecoveryStop as Stop,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::residency::{
    kind_commitment::FullProjectionEncoding, kind_projection::full::FullKindProjection,
    RegionContentFailure,
};
use crate::symbols::data::SymbolTextLookup;
use serde::Serialize;
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

pub(in crate::storage::residency) struct Captured {
    pub(in crate::storage::residency) head:
        (RelationalDurableMemberLocator, ExecutionImmutableBytes),
    pub(in crate::storage::residency) history:
        (RelationalDurableMemberLocator, ExecutionImmutableBytes),
    pub(in crate::storage::residency) kind_authority: super::super::RelationalRegionKindCommitment,
    pub(in crate::storage::residency) history_members: Arc<RelationalHistoryMemberDirectory>,
}
fn invalid(detail: &'static str) -> Stop {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail).into()
}
fn preparation(cause: RegionContentFailure) -> Stop {
    match cause {
        RegionContentFailure::Execution(cause) => Stop::Request(cause),
        RegionContentFailure::Content(
            crate::storage::overlay::PartitionContentDigestError::Request(cause),
        ) => Stop::Request(cause),
        RegionContentFailure::Storage(failure) => failure.into(),
        _ => invalid("original Full region projection refused"),
    }
}

pub(in crate::storage::residency) fn capture(
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
    schema: &RelationalBranchRootSchemaAuthority,
    schema_digest: [u8; 32],
    _symbols: &dyn SymbolTextLookup,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Captured, Stop> {
    for index in 0..partition.entity_arena.history_prefixes.len() {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        if partition.entity_arena.history_prefixes[index].is_some() {
            return Err(invalid(
                "Complete emission requires prefix-free Full entity rows",
            ));
        }
    }
    for index in 0..partition.relation_arena.history_prefixes.len() {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        if partition.relation_arena.history_prefixes[index].is_some() {
            return Err(invalid(
                "Complete emission requires prefix-free Full relation rows",
            ));
        }
    }
    let contracts = aspects::Contracts::capture(schema, request)?;
    arena::validate(&partition.entity_arena, schema, &contracts, request)?;
    arena::validate(&partition.relation_arena, schema, &contracts, request)?;
    let head = FullKindProjection::capture(partition, AdjacencyKindBasis::Current, request)
        .map_err(preparation)?;
    let history = FullKindProjection::capture(partition, AdjacencyKindBasis::Historical, request)
        .map_err(preparation)?;
    let kind_authority = super::super::RelationalRegionKindCommitment {
        head: super::super::kind_commitment::digest_full(&head, request).map_err(preparation)?,
        history: super::super::kind_commitment::digest_full(&history, request)
            .map_err(preparation)?,
    };
    let head_projection = FullProjectionEncoding::new(&head, *request);
    let history_projection = FullProjectionEncoding::new(&history, *request);
    let history_wire = History {
        kind_projection: &history_projection,
        adjacency_kinds: adjacency::Kinds(&partition.adjacency, AdjacencyKindBasis::Historical),
        reverse_adjacency_kinds: adjacency::Kinds(
            &partition.reverse_adjacency,
            AdjacencyKindBasis::Historical,
        ),
        entities: super::super::history_capture::complete::Rows(&partition.entity_arena),
        relations: super::super::history_capture::complete::Rows(&partition.relation_arena),
    };
    let encoded = RelationalRegionImage::encode_for_request(
        partition.partition_id,
        schema_digest,
        RelationalRegionImageKind::History,
        &history_wire,
        request,
    );
    if let Some(cause) = history_projection.take_stop() {
        return Err(Stop::Request(cause));
    }
    let history = encoded?;
    let head_wire = Head {
        kind_projection: &head_projection,
        history_kind_projection: &history_projection,
        partition: Partition(partition),
        history_frontiers: commitment.borrowed_history_frontiers(partition),
        adjacency_kinds: adjacency::Kinds(&partition.adjacency, AdjacencyKindBasis::Current),
        reverse_adjacency_kinds: adjacency::Kinds(
            &partition.reverse_adjacency,
            AdjacencyKindBasis::Current,
        ),
    };
    let encoded = RelationalRegionImage::encode_for_request(
        partition.partition_id,
        schema_digest,
        RelationalRegionImageKind::Head,
        &head_wire,
        request,
    );
    if let Some(cause) = head_projection
        .take_stop()
        .or_else(|| history_projection.take_stop())
    {
        return Err(Stop::Request(cause));
    }
    let head = encoded?;
    Ok(Captured {
        head,
        history,
        kind_authority,
        history_members: RelationalHistoryMemberDirectory::empty_for_request(request)?,
    })
}
#[derive(Serialize)]
struct Head<'a, P: Serialize, H: Serialize, F: Serialize> {
    kind_projection: P,
    history_kind_projection: H,
    partition: Partition<'a>,
    history_frontiers: F,
    adjacency_kinds: adjacency::Kinds<'a>,
    reverse_adjacency_kinds: adjacency::Kinds<'a>,
}
#[derive(Serialize)]
struct History<'a, P: Serialize> {
    kind_projection: P,
    adjacency_kinds: adjacency::Kinds<'a>,
    reverse_adjacency_kinds: adjacency::Kinds<'a>,
    entities: super::super::history_capture::complete::Rows<
        'a,
        crate::storage::substrate::EntityRecordKind,
    >,
    relations: super::super::history_capture::complete::Rows<
        'a,
        crate::storage::substrate::RelationRecordKind,
    >,
}
struct Partition<'a>(&'a PartitionState);
impl Serialize for Partition<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut fields = serializer.serialize_struct("PartitionCheckpointImage", 5)?;
        fields.serialize_field("partition_id", &self.0.partition_id)?;
        fields.serialize_field("entity_arena", &arena::Arena(&self.0.entity_arena))?;
        fields.serialize_field("relation_arena", &arena::Arena(&self.0.relation_arena))?;
        fields.serialize_field("adjacency", &adjacency::Table(&self.0.adjacency))?;
        fields.serialize_field(
            "reverse_adjacency",
            &adjacency::Table(&self.0.reverse_adjacency),
        )?;
        fields.end()
    }
}
