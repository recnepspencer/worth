//! Reached native kind-contract readmission and actual retiring effect grammar.
use super::*;
use crate::schema::data::{RelationalSchemaRegistry, SchemaAuthoritySnapshot};
use std::io::Write;

pub(super) fn reserve(
    kinds: &Arc<Vec<CanonicalCodecRetirementKind>>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<worth_execution::ExecutionMemoryReservation, Stop> {
    // No temporary byte buffer: charge the actual metadata traversal as encoded.
    let mut width = Width {
        context,
        bytes: 0,
        stopped: None,
    };
    let encoded = rmp_serde::encode::write_named(&mut width, kinds.as_ref());
    if let Some(stop) = width.stopped {
        return Err(request(stop));
    }
    encoded.map_err(|_| Stop::from("codec kind metadata width refused"))?;
    // Portable fields, readmitted contracts, canonical registries and lowered
    // plans overlap. Sixty-four bytes per encoded metadata byte bounds these
    // concrete node/header representations, including BTree/Arc slack.
    let bytes = width
        .bytes
        .checked_mul(64)
        .and_then(|n| n.checked_add(kinds.len().checked_mul(512)?))
        .ok_or_else(super::super::memory_overflow)?;
    super::super::reserve(lease, bytes)
}

struct Width<'a, 'b, 'c> {
    context: &'a mut MapKernelContext<'b, 'c>,
    bytes: usize,
    stopped: Option<crate::execution::RelationalExecutionDenialCause>,
}
impl Write for Width<'_, '_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Err(stop) = self.context.checkpoint(bytes.len() as u64) {
            self.stopped = Some(stop.into());
            return Err(std::io::Error::other("codec metadata request stopped"));
        }
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("codec metadata width overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn readmit_kinds(
    event: &CanonicalRequiredOutputCodecRetirement,
    source: &SchemaAuthoritySnapshot,
    target: &SchemaAuthoritySnapshot,
) -> Result<(RelationalSchemaRegistry, RelationalSchemaRegistry), Stop> {
    let mut before = RelationalSchemaRegistry::new();
    let mut after = RelationalSchemaRegistry::new();
    for kind in event.kinds.iter() {
        before = before
            .register_entity_kind(kind.source.readmit()?)
            .map_err(|_| "codec source kind readmission refused")?;
        after = after
            .register_entity_kind(kind.target.readmit()?)
            .map_err(|_| "codec target kind readmission refused")?;
    }
    let observed_source = before.authority_snapshot();
    let observed_target = after.authority_snapshot();
    if observed_source
        .entity_kinds
        .iter()
        .any(|kind| !source.entity_kinds.contains(kind))
        || observed_target
            .entity_kinds
            .iter()
            .any(|kind| !target.entity_kinds.contains(kind))
    {
        return Err(
            "codec disposition kind contracts differ from actual native schema authorities".into(),
        );
    }
    Ok((before, after))
}

pub(super) fn require_delete<'a>(
    envelope: &'a CanonicalCommitEnvelope,
    target: RecordRef,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<&'a crate::publication::patch::data::PublishedAuthoritativeRecordPatch, Stop> {
    use crate::publication::patch::data::RecordStructuralChange as Change;
    charge(
        context,
        envelope
            .patch
            .authoritative_record_patches
            .len()
            .saturating_mul(2)
            .saturating_add(envelope.merged_plan.merged_intents.len()),
    )?;
    if envelope
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|patch| patch.target == target)
        .count()
        != 1
    {
        return Err("codec disposition requires one exact native retirement patch".into());
    }
    let patch = envelope
        .patch
        .authoritative_record_patches
        .iter()
        .find(|patch| patch.target == target)
        .ok_or("codec disposition lacks actual native retirement")?;
    if patch.structural_change != Change::Deleted || patch.contains_opaque_aspect {
        return Err("codec disposition lacks exact classified native deletion".into());
    }
    let actual = envelope
        .merged_plan
        .merged_intents
        .iter()
        .filter(|intent| match (&target, intent) {
            (
                RecordRef::Entity(id),
                MutationIntent::Entity(EntityMutationIntent::Delete(delete)),
            ) => *id == delete.entity_id,
            (
                RecordRef::Relation(id),
                MutationIntent::Relation(RelationMutationIntent::Delete(delete)),
            ) => *id == delete.relation_id,
            _ => false,
        })
        .count();
    if actual != 1 {
        return Err("codec disposition retirement differs from actual native operation".into());
    }
    Ok(patch)
}
