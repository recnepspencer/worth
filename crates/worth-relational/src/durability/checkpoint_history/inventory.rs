//! One required, shared canonical member roster; clones keep its original bill.
use crate::{
    durability::medium::RelationalDurableMemberLocator,
    publication::patch::data::PatchStreamPosition,
};
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalCheckpointMember {
    pub(crate) position: PatchStreamPosition,
    pub(crate) member: RelationalDurableMemberLocator,
}
#[derive(Debug)]
struct Backing {
    entries: Box<[CanonicalCheckpointMember]>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CanonicalCheckpointMembers(Option<Arc<Backing>>);
impl CanonicalCheckpointMembers {
    pub(crate) fn entries(&self) -> &[CanonicalCheckpointMember] {
        self.0.as_ref().map_or(&[], |owner| &owner.entries)
    }
    pub(crate) fn member(
        &self,
        position: PatchStreamPosition,
    ) -> Option<RelationalDurableMemberLocator> {
        let offset = usize::try_from(position.0.checked_sub(1)?).ok()?;
        self.entries()
            .get(offset)
            .filter(|row| row.position == position)
            .map(|row| row.member)
    }
    pub(crate) fn allocation_bytes(count: usize) -> Option<u64> {
        if count == 0 {
            return Some(0);
        }
        let control = Layout::new::<(usize, usize)>()
            .extend(Layout::new::<Backing>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(
            control.size().checked_add(
                Layout::array::<CanonicalCheckpointMember>(count)
                    .ok()?
                    .size(),
            )?,
        )
        .ok()
    }
    pub(crate) fn reserve(
        count: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<ExecutionMemoryReservation>, crate::execution::RelationalExecutionDenialCause>
    {
        if count == 0 {
            request
                .check_live()
                .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
            return Ok(None);
        }
        let size = Self::allocation_bytes(count)
            .ok_or(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)?;
        request.reserve_retained_memory(size).map_err(Into::into)
    }
    pub(crate) fn from_reserved(
        entries: Vec<CanonicalCheckpointMember>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        if entries.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Backing {
            entries: entries.into_boxed_slice(),
            _memory: memory,
        })))
    }
    pub(crate) fn attach(
        &mut self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        let Some(owner) = &mut self.0 else {
            return Ok(());
        };
        let Some(owner) = Arc::get_mut(owner).filter(|owner| owner._memory.is_none()) else {
            return Err(memory);
        };
        owner._memory = memory;
        Ok(())
    }
}
impl PartialEq for CanonicalCheckpointMembers {
    fn eq(&self, other: &Self) -> bool {
        self.entries() == other.entries()
    }
}
impl Eq for CanonicalCheckpointMembers {}
impl Serialize for CanonicalCheckpointMembers {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.entries().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CanonicalCheckpointMembers {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Roster;
        impl<'de> serde::de::Visitor<'de> for Roster {
            type Value = CanonicalCheckpointMembers;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("the complete ordered canonical member sequence")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let count = seq.size_hint().ok_or_else(|| {
                    serde::de::Error::custom("canonical roster length is required")
                })?;
                let mut rows = Vec::with_capacity(count);
                while let Some(row) = seq.next_element()? {
                    if rows.len() == count {
                        return Err(serde::de::Error::custom(
                            "canonical roster exceeds declared length",
                        ));
                    }
                    rows.push(row);
                }
                if rows.len() != count {
                    return Err(serde::de::Error::custom("canonical roster is truncated"));
                }
                Ok(CanonicalCheckpointMembers::from_reserved(rows, None))
            }
        }
        deserializer.deserialize_seq(Roster)
    }
}

pub(crate) fn validate(
    manifest: &crate::durability::data::RelationalCheckpointManifestDraft,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), crate::durability::data::RelationalRecoveryStop> {
    use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
    let malformed = || {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "canonical member inventory is not the exact indexed prefix",
        )
    };
    let rows = manifest.canonical_members.entries();
    if rows.len() != manifest.commits.len() {
        return Err(malformed().into());
    }
    let scratch = std::alloc::Layout::array::<RelationalDurableMemberLocator>(rows.len())
        .map_err(|_| malformed())?;
    let _memory = request
        .reserve_retained_memory(scratch.size() as u64)
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
    let mut members = Vec::with_capacity(rows.len());
    for (ordinal, (row, entry)) in rows.iter().zip(&manifest.commits).enumerate() {
        request
            .consult()
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        if row.position != entry.position
            || row.position.0 != ordinal as u64 + 1
            || row.member.length < 66
        {
            return Err(malformed().into());
        }
        members.push(row.member);
    }
    members.sort_unstable();
    if members.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(malformed().into());
    }
    let canonical = |member| members.binary_search(&member).is_ok();
    if canonical(manifest.derived_indexes)
        || manifest
            .roots
            .iter()
            .flat_map(|root| &root.regions)
            .chain(&manifest.mirror)
            .any(|leaf| canonical(leaf.head) || canonical(leaf.history))
        || manifest.history_dependencies.iter().any(|dependency| {
            canonical(dependency.member) || dependency.bases.iter().any(|member| canonical(*member))
        })
        || manifest
            .symbols
            .iter()
            .chain(&manifest.lineage)
            .any(|page| canonical(page.member))
        || manifest
            .symbol_buckets
            .iter()
            .any(|bucket| bucket.pages.iter().any(|member| canonical(*member)))
        || manifest
            .lineage_events
            .iter()
            .any(|page| canonical(page.member))
    {
        return Err(malformed().into());
    }
    Ok(())
}

pub(crate) fn prepare_decode(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, crate::durability::data::RelationalRecoveryStop> {
    use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal as Refusal;
    use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
    let mut roster = None;
    let mut manifest_seen = false;
    let scanned = BorrowedPayload(payload).fields(request, |name, value| {
        if name == "manifest" {
            if manifest_seen {
                return Err(Refusal::PayloadDecode {
                    detail: "duplicate checkpoint manifest".into(),
                });
            }
            manifest_seen = true;
            value.fields(request, |name, value| {
                if name == "canonical_members" {
                    if roster.is_some() {
                        return Err(Refusal::PayloadDecode {
                            detail: "duplicate canonical member inventory".into(),
                        });
                    }
                    roster = Some(value.elements(request, |_| Ok(()))?);
                }
                Ok(())
            })?;
        }
        Ok(())
    });
    scanned.map_err(|cause| match cause {
        Refusal::Request(cause) => RelationalRecoveryStop::Request(cause),
        cause => DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("canonical inventory preflight: {cause:?}"),
        )
        .into(),
    })?;
    let count = roster.ok_or_else(|| {
        DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "canonical member inventory is required",
        )
    })?;
    CanonicalCheckpointMembers::reserve(count, request).map_err(RelationalRecoveryStop::Request)
}

#[cfg(test)]
mod tests;
