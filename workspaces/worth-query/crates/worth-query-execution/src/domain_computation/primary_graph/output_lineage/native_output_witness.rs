//! A performed output's native content authority, prepared before publication.

use std::mem::size_of;
use std::sync::{Arc, OnceLock};

use worth_foundational::facade::AspectKey;
use worth_relational::facade::{
    identity::{EntityId, KindId, VersionId},
    mvcc::CompanionPreflightStop,
    runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputPosture,
    },
    provider::WorthQueryPrimaryGraphApplicationAttempt,
    schema_layout::WorthQueryPrimaryGraphLayout,
};

use super::invalidation::{InvalidationEditAdmission, SourceInvalidationOwner};
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;

mod checkpoint;
mod equality;
mod fact_coverage;
mod preparation;
mod retirement;
use retirement::RetirementMetadata;

#[derive(Debug)]
struct RoleProbe {
    role: String,
    entity_name: String,
    kind: KindId,
    posture: WorthQueryApplicationOutputPosture,
    retirement: Option<RetirementMetadata>,
    entity: Option<EntityId>,
    first_aspect: usize,
    end_aspect: usize,
}

#[derive(Debug)]
struct AspectProbe {
    aspect: AspectKey,
    // Outer None means the performed native basis has not filled this slot.
    revision: Option<Option<u64>>,
}

/// Its two Vec backings and the final Arc cell are admitted before the native
/// commit. Neither this ticket nor its sealed successor authorizes a write.
#[must_use = "the prepared output witness must be sealed or dropped"]
pub(in crate::domain_computation::primary_graph) struct PreparedNativeOutputWitness {
    cell: Arc<OnceLock<SealedNativeOutputWitness>>,
    roles: Vec<RoleProbe>,
    aspects: Vec<AspectProbe>,
    retained: Arc<RetainedInvalidationCapacity>,
}

/// Equality of all installed aspect revisions plus native liveness/generation
/// proves that the previously performed output has not changed. A revision
/// change conservatively requests ordinary producer execution, even for ABA.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct SealedNativeOutputWitness {
    roles: Vec<RoleProbe>,
    aspects: Vec<AspectProbe>,
    _retained: Arc<RetainedInvalidationCapacity>,
}

/// A checked projection of the sealed performed roles into ordinary observed
/// facts. It borrows the witness so the installed aspect list remains the only
/// source of this output dependency footprint.
pub(in crate::domain_computation::primary_graph) struct PreparedNativeOutputFacts<'a> {
    witness: &'a SealedNativeOutputWitness,
    count: usize,
    retained_payload_bytes: u64,
}

impl PreparedNativeOutputFacts<'_> {
    pub(in crate::domain_computation::primary_graph) fn count(&self) -> usize {
        self.count
    }

    pub(in crate::domain_computation::primary_graph) fn retained_payload_bytes(&self) -> u64 {
        self.retained_payload_bytes
    }

    /// The caller has already reserved the complete flat Vec and Arc. No
    /// additional allocation is possible except the admitted aspect clones.
    pub(in crate::domain_computation::primary_graph) fn append_into(self, facts: &mut Vec<Fact>) {
        assert!(facts.capacity().saturating_sub(facts.len()) >= self.count);
        for role in &self.witness.roles {
            let entity_id = role.entity.expect("prepared witness role is filled");
            facts.push(Fact::Entity {
                entity_id,
                kind: role.kind,
            });
            for aspect in &self.witness.aspects[role.first_aspect..role.end_aspect] {
                facts.push(Fact::SourceAspectRevision {
                    entity_id,
                    aspect: aspect.aspect.clone(),
                    native_revision: aspect.revision.expect("prepared aspect revision is filled"),
                });
            }
        }
    }
}

fn overflow() -> CompanionPreflightStop {
    CompanionPreflightStop::PreparationMemoryCounterOverflow
}

impl PreparedNativeOutputWitness {
    /// The committed correspondence supplies created IDs. All storage was
    /// allocated by `prepare`; an unavailable native projection only makes
    /// this optional reuse proof ineligible.
    pub(in crate::domain_computation::primary_graph) fn finish(
        mut self,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
        runtime: &RelationalRuntime,
        committed: &SnapshotHandle,
    ) -> Option<Arc<OnceLock<SealedNativeOutputWitness>>> {
        let read = runtime.read_truth();
        let actual = correspondence.native_witness_roles();
        if actual.len() != self.roles.len() {
            return None;
        }
        for (role, (name, posture, entity_name, entity)) in self.roles.iter_mut().zip(actual) {
            if role.role != name || role.entity_name != entity_name || posture != role.posture {
                return None;
            }
            if role.posture == WorthQueryApplicationOutputPosture::Retire {
                let metadata = retirement::native_metadata(runtime, committed, entity)?;
                if metadata.kind != role.kind || metadata.deleted_at != committed.version_id() {
                    return None;
                }
                role.retirement = Some(metadata);
            } else if read.exact_snapshot_live_entity_kind(committed, entity) != Some(role.kind) {
                return None;
            }
            role.entity = Some(entity);
            for aspect in &mut self.aspects[role.first_aspect..role.end_aspect] {
                aspect.revision = Some(read.exact_snapshot_entity_aspect_version(
                    committed,
                    entity,
                    &aspect.aspect,
                )?);
            }
        }
        let sealed = SealedNativeOutputWitness {
            roles: self.roles,
            aspects: self.aspects,
            _retained: self.retained,
        };
        self.cell.set(sealed).ok()?;
        Some(self.cell)
    }
}

impl SealedNativeOutputWitness {
    /// Claim the actual role/aspect traversal and initialized name copies
    /// before a stable alias materializes its observed-fact row. An unfilled
    /// witness cannot be promoted into reusable output authority.
    pub(in crate::domain_computation::primary_graph) fn prepare_fact_projection(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<PreparedNativeOutputFacts<'_>>, CompanionPreflightStop> {
        let mut count = 0usize;
        let mut payload = 0u64;
        for role in &self.roles {
            admission.charge_external_work(1)?;
            if role.entity.is_none() {
                return Ok(None);
            }
            count = count
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            // Descriptive checkpoint facts cannot prove the original producer
            // publication for a dead member. Keep the existing Fresh posture.
            if role.posture == WorthQueryApplicationOutputPosture::Retire {
                return Ok(None);
            }
            for aspect in &self.aspects[role.first_aspect..role.end_aspect] {
                admission.charge_external_work(1)?;
                if aspect.revision.is_none() {
                    return Ok(None);
                }
                let initialized = u64::try_from(aspect.aspect.as_str().len())
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
                admission.charge_external_work(initialized)?;
                payload = payload
                    .checked_add(
                        u64::try_from(aspect.aspect.owned_allocation_capacity_bytes()).map_err(
                            |_| CompanionPreflightStop::PreparationMemoryCounterOverflow,
                        )?,
                    )
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                count = count
                    .checked_add(1)
                    .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            }
        }
        Ok(Some(PreparedNativeOutputFacts {
            witness: self,
            count,
            retained_payload_bytes: payload,
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn unchanged_in(
        &self,
        runtime: &RelationalRuntime,
        selected: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        let read = runtime.read_truth();
        for role in &self.roles {
            admission.charge_external_work(1)?;
            let Some(entity) = role.entity else {
                return Ok(false);
            };
            if role.posture == WorthQueryApplicationOutputPosture::Retire {
                admission.charge_external_work(3)?;
                if role.retirement.is_none()
                    || retirement::native_metadata(runtime, selected, entity) != role.retirement
                {
                    return Ok(false);
                }
                continue;
            }
            if read.exact_snapshot_live_entity_kind(selected, entity) != Some(role.kind) {
                return Ok(false);
            }
            for aspect in &self.aspects[role.first_aspect..role.end_aspect] {
                admission.charge_external_work(aspect.aspect.as_str().len() as u64 + 1)?;
                if read.exact_snapshot_entity_aspect_version(selected, entity, &aspect.aspect)
                    != aspect.revision
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}
