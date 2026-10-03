//! A performed output's native content authority, prepared before publication.

use std::mem::size_of;
use std::sync::{Arc, OnceLock};

use worth_foundational::facade::AspectKey;
use worth_relational::facade::{
    identity::{EntityId, KindId},
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

#[derive(Debug)]
struct RoleProbe {
    role: String,
    entity_name: String,
    kind: KindId,
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
    pub(in crate::domain_computation::primary_graph) fn prepare(
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        layout: &WorthQueryPrimaryGraphLayout,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, CompanionPreflightStop> {
        let roles = attempt.native_witness_roles();
        let role_count = roles.len();
        if role_count == 0 {
            return Ok(None);
        }
        admission.charge_external_work(role_count as u64)?;
        let mut aspect_count = 0usize;
        let mut name_bytes = 0u64;
        let mut catalog_work = 0u64;
        for (role, posture, entity_name) in roles.clone() {
            if posture == WorthQueryApplicationOutputPosture::Retire {
                return Ok(None);
            }
            let name_lookup_work = layout
                .entity_kind_lookup_work(entity_name)
                .ok_or_else(overflow)?;
            let comparison_bytes =
                u64::try_from(entity_name.len().checked_add(1).ok_or_else(overflow)?)
                    .map_err(|_| overflow())?;
            let one_scan_work = u64::try_from(layout.native_contract_count())
                .ok()
                .and_then(|count| count.checked_mul(comparison_bytes))
                .ok_or_else(overflow)?;
            admission.charge_external_work(
                one_scan_work
                    .checked_add(name_lookup_work)
                    .ok_or_else(overflow)?,
            )?;
            if layout.entity_kind(entity_name).is_none() {
                return Ok(None);
            }
            name_bytes = name_bytes
                .checked_add(role.len() as u64)
                .and_then(|n| n.checked_add(entity_name.len() as u64))
                .ok_or_else(overflow)?;
            catalog_work = catalog_work
                .checked_add(one_scan_work)
                .and_then(|work| work.checked_add(name_lookup_work))
                .ok_or_else(overflow)?;
            for aspect in layout.native_output_aspects(entity_name) {
                aspect_count = aspect_count.checked_add(1).ok_or_else(overflow)?;
                name_bytes = name_bytes
                    .checked_add(aspect.as_str().len() as u64)
                    .ok_or_else(overflow)?;
            }
        }
        let backing = role_count
            .checked_mul(size_of::<RoleProbe>())
            .and_then(|bytes| {
                bytes.checked_add(aspect_count.checked_mul(size_of::<AspectProbe>())?)
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(name_bytes))
            .ok_or_else(overflow)?;
        // A second catalog walk builds the owned probes. Later performed-root
        // reads and canonical role comparisons are also paid before cutover.
        let visits = role_count
            .checked_mul(
                layout
                    .native_contract_count()
                    .checked_add(2)
                    .ok_or_else(overflow)?,
            )
            .and_then(|count| count.checked_add(aspect_count))
            .ok_or_else(overflow)? as u64;
        // Vec slot widths and the Arc cell are retained memory. Constructing
        // each initialized row is one visit; only owned names copy UTF-8.
        let copy_work = name_bytes.checked_add(visits).ok_or_else(overflow)?;
        let copy_work = copy_work.checked_add(catalog_work).ok_or_else(overflow)?;
        let copy_work = copy_work.checked_add(name_bytes).ok_or_else(overflow)?;
        let retained = owner.retain_native_output_witness::<SealedNativeOutputWitness>(
            backing, copy_work, admission,
        )?;
        let cell = Arc::new(OnceLock::new());
        let mut prepared = Self {
            cell,
            roles: Vec::with_capacity(role_count),
            aspects: Vec::with_capacity(aspect_count),
            retained,
        };
        for (role, _, entity_name) in roles {
            let first_aspect = prepared.aspects.len();
            for aspect in layout.native_output_aspects(entity_name) {
                prepared.aspects.push(AspectProbe {
                    aspect: aspect.clone(),
                    revision: None,
                });
            }
            prepared.roles.push(RoleProbe {
                role: role.to_owned(),
                entity_name: entity_name.to_owned(),
                kind: layout
                    .entity_kind(entity_name)
                    .expect("counted installed output kind"),
                entity: None,
                first_aspect,
                end_aspect: prepared.aspects.len(),
            });
        }
        Ok(Some(prepared))
    }

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
            if role.role != name
                || role.entity_name != entity_name
                || posture == WorthQueryApplicationOutputPosture::Retire
                || read.exact_snapshot_live_entity_kind(committed, entity) != Some(role.kind)
            {
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
