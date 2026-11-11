//! Relation projections bound to the selected manifest's root commitment.
//! Member hashing alone cannot replace the leaf's admitted projection digest.
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::RegionRelationProjection;
use crate::storage::substrate::maintenance::ReconstructionLane;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RelationalRegionKindCommitment {
    pub(crate) head: worth_foundational::CanonicalDigestId,
    pub(crate) history: worth_foundational::CanonicalDigestId,
}

#[cfg(test)]
pub(super) fn digest(
    lane: &ReconstructionLane<'_>,
    projection: &[RegionRelationProjection],
) -> Result<worth_foundational::CanonicalDigestId, Cause> {
    digest_controlled(lane, projection, &mut |_, _| Ok(()))
}

pub(super) fn digest_full(
    lane: &ReconstructionLane<'_>,
    projection: &super::kind_projection::full::FullKindProjection,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<worth_foundational::CanonicalDigestId, super::RegionContentFailure> {
    projection.commitment(lane, request)
}

type Stop = std::cell::Cell<Option<crate::execution::RelationalExecutionDenialCause>>;
fn poll_encoding<E: serde::ser::Error>(
    request: worth_execution::ExecutionRequest<'_, '_>,
    stopped: &Stop,
) -> Result<(), E> {
    request.consult().map_err(|cause| {
        stopped.set(Some(cause.into()));
        E::custom("original region preparation request stopped")
    })
}

pub(in crate::storage::residency) struct FullProjectionEncoding<'a, 'request, 'authority> {
    projection: &'a super::kind_projection::full::FullKindProjection,
    request: worth_execution::ExecutionRequest<'request, 'authority>,
    stopped: Stop,
}
impl Serialize for FullProjectionEncoding<'_, '_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut count = 0usize;
        let mut previous = None;
        for (&(relation, _, _), _) in self.projection.rows().iter() {
            poll_encoding::<S::Error>(self.request, &self.stopped)?;
            if previous != Some(relation) {
                count += 1;
                previous = Some(relation);
            }
        }
        let mut sequence = serializer.serialize_seq(Some(count))?;
        let mut rows = self.projection.rows().iter().peekable();
        while let Some((&(relation, _, _), &kind)) = rows.next() {
            poll_encoding::<S::Error>(self.request, &self.stopped)?;
            sequence.serialize_element(&RelationEncoding {
                projection: self.projection,
                request: self.request,
                stopped: &self.stopped,
                relation,
                kind: crate::identity::data::KindId(kind),
            })?;
            while rows.peek().is_some_and(|(key, _)| key.0 == relation) {
                poll_encoding::<S::Error>(self.request, &self.stopped)?;
                rows.next();
            }
        }
        sequence.end()
    }
}

struct RelationEncoding<'a, 'request, 'authority> {
    projection: &'a super::kind_projection::full::FullKindProjection,
    request: worth_execution::ExecutionRequest<'request, 'authority>,
    stopped: &'a Stop,
    relation: crate::identity::data::RelationId,
    kind: crate::identity::data::KindId,
}
impl Serialize for RelationEncoding<'_, '_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut fields = serializer.serialize_struct("RegionRelationProjection", 4)?;
        fields.serialize_field("relation", &self.relation)?;
        fields.serialize_field("kind", &self.kind)?;
        for (name, incoming) in [("outgoing", false), ("incoming", true)] {
            fields.serialize_field(
                name,
                &SlotsEncoding {
                    relation: self.relation,
                    incoming,
                    projection: self.projection,
                    request: self.request,
                    stopped: self.stopped,
                },
            )?;
        }
        fields.end()
    }
}

struct SlotsEncoding<'a, 'request, 'authority> {
    projection: &'a super::kind_projection::full::FullKindProjection,
    request: worth_execution::ExecutionRequest<'request, 'authority>,
    stopped: &'a Stop,
    relation: crate::identity::data::RelationId,
    incoming: bool,
}
impl SlotsEncoding<'_, '_, '_> {
    fn next_slot(&self, start: usize) -> Option<usize> {
        let (&(relation, incoming, slot), _) =
            self.projection
                .rows()
                .lower_bound(&(self.relation, self.incoming, start))?;
        (relation == self.relation && incoming == self.incoming).then_some(slot)
    }
}
impl Serialize for SlotsEncoding<'_, '_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut count = 0usize;
        let mut start = Some(0);
        while let Some(slot) = start.and_then(|start| self.next_slot(start)) {
            poll_encoding::<S::Error>(self.request, self.stopped)?;
            count += 1;
            start = slot.checked_add(1);
        }
        let mut sequence = serializer.serialize_seq(Some(count))?;
        let mut start = Some(0);
        while let Some(slot) = start.and_then(|start| self.next_slot(start)) {
            poll_encoding::<S::Error>(self.request, self.stopped)?;
            sequence.serialize_element(&slot)?;
            start = slot.checked_add(1);
        }
        sequence.end()
    }
}

pub(super) fn digest_controlled(
    lane: &ReconstructionLane<'_>,
    projection: &[RegionRelationProjection],
    checkpoint: &mut super::kind_projection::ProjectionCheckpoint<'_>,
) -> Result<worth_foundational::CanonicalDigestId, Cause> {
    checkpoint(1, 0)?;
    super::kind_index::KindIndex::rebuild(lane, projection, checkpoint)
        .map(|index| worth_foundational::CanonicalDigestId::new(index.digest()))
}

pub(super) fn verify(
    lane: &ReconstructionLane<'_>,
    projection: &[RegionRelationProjection],
    expected: worth_foundational::CanonicalDigestId,
    partition: crate::identity::data::PartitionId,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), super::RelationalResidencyDemandFailure> {
    use super::region_load::materialization_funding as funding;
    let mut memory = funding::Funding::default();
    let mut stopped = None;
    let found = digest_controlled(lane, projection, &mut |units, bytes| match funding::work(
        request, units,
    )
    .and_then(|()| memory.grow(bytes, request))
    {
        Ok(()) => Ok(()),
        Err(cause) => {
            stopped = Some(cause);
            Err(Cause::SourceUnavailable)
        }
    });
    if let Some(cause) = stopped {
        return Err(cause);
    }
    let unavailable = |cause| {
        super::RelationalResidencyDemandFailure::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
                cause,
            },
        )
    };
    if found.map_err(unavailable)? != expected {
        return Err(unavailable(Cause::IntegrityMismatch));
    }
    Ok(())
}

#[cfg(test)]
impl RelationalRegionKindCommitment {
    pub(crate) fn capture(
        lane: &ReconstructionLane<'_>,
        partition: &crate::storage::overlay::PartitionState,
    ) -> Result<Self, Cause> {
        use crate::storage::partition::AdjacencyKindBasis;
        Ok(Self {
            head: digest(
                lane,
                &super::kind_projection::capture(lane, partition, AdjacencyKindBasis::Current)?,
            )?,
            history: digest(
                lane,
                &super::kind_projection::capture(lane, partition, AdjacencyKindBasis::Historical)?,
            )?,
        })
    }
}

impl<'a, 'request, 'authority> FullProjectionEncoding<'a, 'request, 'authority> {
    pub(in crate::storage::residency) fn new(
        projection: &'a super::kind_projection::full::FullKindProjection,
        request: worth_execution::ExecutionRequest<'request, 'authority>,
    ) -> Self {
        Self {
            projection,
            request,
            stopped: Stop::new(None),
        }
    }
    pub(in crate::storage::residency) fn take_stop(
        &self,
    ) -> Option<crate::execution::RelationalExecutionDenialCause> {
        self.stopped.take()
    }
}
