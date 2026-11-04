//! Relation projections bound to the selected manifest's root commitment.
//! Member hashing alone cannot replace the leaf's admitted projection digest.
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::RegionRelationProjection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RelationalRegionKindCommitment {
    pub(crate) head: worth_foundational::CanonicalDigestId,
    pub(crate) history: worth_foundational::CanonicalDigestId,
}

pub(super) fn digest(
    projection: &[RegionRelationProjection],
) -> Result<worth_foundational::CanonicalDigestId, Cause> {
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.region-kind-authority.v2\0");
    rmp_serde::encode::write(&mut KindHashWriter(&mut hash), projection)
        .map_err(|_| Cause::Damaged)?;
    Ok(worth_foundational::CanonicalDigestId::new(
        hash.finalize().into(),
    ))
}

struct KindHashWriter<'a>(&'a mut Sha256);
impl std::io::Write for KindHashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn digest_full(
    projection: &super::kind_projection::full::FullKindProjection,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<worth_foundational::CanonicalDigestId, super::RegionContentFailure> {
    let encoding = FullProjectionEncoding {
        projection,
        request: *request,
        stopped: std::cell::Cell::new(None),
    };
    let mut hash = Sha256::new();
    hash.update(b"worth.relational.region-kind-authority.v2\0");
    let result = rmp_serde::encode::write(&mut KindHashWriter(&mut hash), &encoding);
    if let Some(cause) = encoding.stopped.take() {
        return Err(super::RegionContentFailure::Request(cause));
    }
    result.map_err(|_| super::RegionContentFailure::Adjacency)?;
    Ok(worth_foundational::CanonicalDigestId::new(
        hash.finalize().into(),
    ))
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

pub(super) fn verify(
    projection: &[RegionRelationProjection],
    expected: worth_foundational::CanonicalDigestId,
) -> Result<(), Cause> {
    if digest(projection)? != expected {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}

impl RelationalRegionKindCommitment {
    pub(crate) fn capture(
        partition: &crate::storage::overlay::PartitionState,
    ) -> Result<Self, Cause> {
        use crate::storage::partition::AdjacencyKindBasis;
        Ok(Self {
            head: digest(&super::kind_projection::capture(
                partition,
                AdjacencyKindBasis::Current,
            )?)?,
            history: digest(&super::kind_projection::capture(
                partition,
                AdjacencyKindBasis::Historical,
            )?)?,
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
