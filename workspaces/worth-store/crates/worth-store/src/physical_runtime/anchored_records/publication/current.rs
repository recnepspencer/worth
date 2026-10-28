//! Deciding a request against the anchor's current value, before any effect:
//! an identical retry, a generation conflict, a full anchor table, or carried
//! members the current value does not list.

use worth_store_physical_format::{RecordAnchor, MAXIMUM_RECORD_ANCHORS};

use super::super::{
    AnchoredRecordMembership, AnchoredRecordName, AnchoredRecordPublicationFailure, AnchoredRecords,
};
use super::admission::AdmittedMembers;

type Failure = AnchoredRecordPublicationFailure;

/// What the current value leaves for a request to do.
pub(super) enum CurrentDecision {
    /// The requested content is already durable at the target generation.
    AlreadyPublished(AnchoredRecordMembership),
    /// The request replaces the current value with generation `target`.
    Replace { target: u64 },
}

/// The payload a request would publish.
#[derive(Debug, Clone, Copy)]
pub(super) struct RequestedPayload {
    pub(super) sha256: [u8; 32],
    pub(super) bytes: u64,
}

impl AnchoredRecords<'_> {
    /// Reads the current anchor, and the current value's verified membership
    /// only when a retry or a carried member needs it. Cost: O(1) without
    /// carried members, else O(log members) once the value is verified.
    pub(super) fn decide_against_current(
        &self,
        name: AnchoredRecordName,
        expected_generation: u64,
        payload: RequestedPayload,
        members: &AdmittedMembers,
    ) -> Result<CurrentDecision, Failure> {
        let reader = self.reader().map_err(Failure::Read)?;
        let anchors = reader.selected_record_anchors();
        let current = anchors.get(name.anchor());
        let current_generation = current.map_or(0, RecordAnchor::generation);
        let Some(target) = expected_generation.checked_add(1) else {
            return Err(Failure::GenerationConflict {
                current: current_generation,
            });
        };
        let current_value = match current {
            Some(anchor) if anchor.generation() == target || members.carries_any() => Some(
                self.membership(&reader, name, anchor)
                    .map_err(Failure::Read)?,
            ),
            _ => None,
        };
        if let Some(value) = current_value.as_ref().filter(|value| {
            value.selected.generation == target
                && value.holds_content(payload.sha256, payload.bytes, members.contents())
        }) {
            return Ok(CurrentDecision::AlreadyPublished(value.described(name)));
        }
        if current_generation != expected_generation {
            return Err(Failure::GenerationConflict {
                current: current_generation,
            });
        }
        if current.is_none() && anchors.iter().count() == MAXIMUM_RECORD_ANCHORS {
            return Err(Failure::AnchorTableFull);
        }
        members.require_carried_current(current_value.as_deref())?;
        Ok(CurrentDecision::Replace { target })
    }
}
