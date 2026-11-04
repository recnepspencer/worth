//! Stream the exact immutable selector once, retaining every allocation grant.
use super::{
    PositionNode, PublicationRoute, RequiredOutputPublicationPositions, RequiredOutputRoutePosition,
};
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use crate::durability::checkpoint_admission::{
    native_stop, RelationalCheckpointAdmissionRefusal as Refusal,
};
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
mod node;

// Even height 64 requires more nodes than the 1 GiB image can hold.
const MAXIMUM_HEIGHT: usize = 64;
type RouteKey<'a> = (&'a str, EntityId, [u8; 32]);

struct Readmission<'r, 'scope, 'authority> {
    request: &'r ExecutionRequest<'scope, 'authority>,
    contains: &'r dyn Fn(CommitId) -> bool,
    stopped: Option<Refusal>,
}
impl Readmission<'_, '_, '_> {
    fn charge(&self, bytes: u64) -> Result<(), Refusal> {
        crate::durability::artifact_payload_emission::request::account_work(self.request, bytes)
            .map_err(native_stop)
    }
    fn stop<E: serde::de::Error>(&mut self, cause: Refusal) -> E {
        self.stopped = Some(cause);
        E::custom("checkpoint route selector refused")
    }
    fn reserve<T>(&self, tail: usize) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .map_err(|_| Refusal::CapacityOverflow)?
            .0
            .pad_to_align();
        let bytes = layout
            .size()
            .checked_add(tail)
            .ok_or(Refusal::CapacityOverflow)? as u64;
        self.charge(bytes)?;
        self.request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Refusal::Request(cause.into()))
    }
}

impl RequiredOutputPublicationPositions {
    pub(crate) fn readmit_checkpoint(
        bytes: &[u8],
        contains: &impl Fn(CommitId) -> bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let mut context = Readmission {
            request,
            contains,
            stopped: None,
        };
        // Two linear passes: exact single-value framing, then streaming decode.
        // Neither framing nor the seed revisits each descendant at every node.
        context.charge(bytes.len() as u64)?;
        BorrowedPayload(bytes)
            .fields(request, |_, _| Ok(()))
            .map_err(projected_stop)?;
        context.charge(bytes.len() as u64)?;
        let mut decoder = rmp_serde::Deserializer::from_read_ref(bytes);
        decoder.set_max_depth(MAXIMUM_HEIGHT + 8);
        let result = Selector(&mut context).deserialize(&mut decoder);
        result.map_err(|_| {
            context
                .stopped
                .take()
                .unwrap_or(Refusal::InvalidProjectedMeaning)
        })
    }
}

struct Selector<'c, 'r, 'scope, 'authority>(&'c mut Readmission<'r, 'scope, 'authority>);
impl<'de> DeserializeSeed<'de> for Selector<'_, '_, '_, '_> {
    type Value = RequiredOutputPublicationPositions;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        decoder.deserialize_struct(
            "CheckpointOutputPositions",
            &["unresolved_prefix", "root"],
            self,
        )
    }
}
impl<'de> Visitor<'de> for Selector<'_, '_, '_, '_> {
    type Value = RequiredOutputPublicationPositions;
    fn expecting(&self, output: &mut std::fmt::Formatter) -> std::fmt::Result {
        output.write_str("the ordered checkpoint route selector")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
        let unresolved_prefix: Option<CommitId> = field(&mut input, "unresolved_prefix")?;
        if unresolved_prefix.is_some_and(|commit| !(self.0.contains)(commit)) {
            return Err(serde::de::Error::custom("unknown unresolved route prefix"));
        }
        key(&mut input, "root")?;
        let mut previous = None;
        let root = input.next_value_seed(node::Node {
            context: self.0,
            previous: &mut previous,
            depth: 0,
        })?;
        finish(&mut input)?;
        Ok(RequiredOutputPublicationPositions {
            root,
            unresolved_prefix,
        })
    }
}

fn key<'de, A: MapAccess<'de>>(input: &mut A, expected: &str) -> Result<(), A::Error> {
    if input.next_key::<&str>()? != Some(expected) {
        return Err(serde::de::Error::custom(
            "checkpoint selector field order or membership",
        ));
    }
    Ok(())
}
fn field<'de, A: MapAccess<'de>, T: serde::Deserialize<'de>>(
    input: &mut A,
    expected: &str,
) -> Result<T, A::Error> {
    key(input, expected)?;
    input.next_value()
}
fn finish<'de, A: MapAccess<'de>>(input: &mut A) -> Result<(), A::Error> {
    if input.next_key::<serde::de::IgnoredAny>()?.is_some() {
        return Err(serde::de::Error::custom("extra checkpoint selector field"));
    }
    Ok(())
}
fn projected_stop(
    cause: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
) -> Refusal {
    match cause {
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Request(cause) => Refusal::Request(cause),
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::Frame(_)
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PayloadDecode { .. }
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PositionDisagreement { .. }
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::CommitIdDisagreement { .. }
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::FieldText(_)
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PositionAdmission(_)
        | crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::NonCanonicalPayload => Refusal::InvalidProjectedMeaning,
    }
}
