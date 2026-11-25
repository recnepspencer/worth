//! Reserve-before-lock admission and exact charging of a bound cell's loads.
//!
//! The demand reserves its estimated pool bytes and in-flight fill bytes
//! before it takes the cell gate (M4 §4.3). Each loaded image then proves its
//! `BasisIdentity` and carries its measured charge inside its own allocation
//! before publication, so a refused charge publishes nothing.
use super::*;
use crate::storage::residency::working_set::{
    BasisIdentity, ResidencyLevelKind, WorkingSetCharge, WorkingSetExhausted,
};

#[derive(Debug, Default)]
pub(in crate::internal_port::storage::residency) struct WorkingSetAdmission {
    prepaid: Option<WorkingSetCharge>,
    _fill: Option<WorkingSetCharge>,
    exhausted: Option<WorkingSetExhausted>,
    reserved_head: bool,
    reserved_full: bool,
}

impl WorkingSetAdmission {
    pub(super) fn covers(&self, need_head: bool, need_full: bool) -> bool {
        if self.exhausted.is_some() {
            return true;
        }
        (!need_head || self.reserved_head) && (!need_full || self.reserved_full)
    }
}

impl BasisIdentity {
    /// The basis a loaded image actually proves, from its sealed commitment.
    pub fn observed(
        partition_id: PartitionId,
        region: &ResidentRegion,
        schema_carrier_digest: [u8; 32],
        level: ResidencyLevelKind,
    ) -> Option<Self> {
        let complete = region.history_presence.is_complete();
        if matches!(level, ResidencyLevelKind::Full) && !complete {
            return None;
        }
        Some(Self {
            content_digest: region.commitment.as_ref()?.digest(partition_id),
            schema_carrier_digest,
            level,
        })
    }
}

impl RelationalRegionResidency {
    /// The exact basis this cell's root leaf names for `level`.
    pub fn expected_basis(&self, level: ResidencyLevelKind) -> Option<BasisIdentity> {
        self.backing.as_ref().map(|cold| BasisIdentity {
            content_digest: cold.content_digest,
            schema_carrier_digest: cold.schema_carrier_digest,
            level,
        })
    }

    fn exhausted(&self) -> Failure {
        Failure::StorageUnavailable(self.failure(Cause::MemoryExhausted))
    }

    /// Pre-lock admission. Unbound cells need none. A refused reservation is
    /// remembered rather than returned: an already resident image still
    /// serves, while any load that would need the bytes refuses before reading.
    pub(super) fn admit_working_set(&self, demand: ResidencyDemand) -> Option<WorkingSetAdmission> {
        let _ = self.working_set.get()?;
        let _ = self.backing.as_ref()?;
        let (need_head, need_full) = match self.state.try_lock() {
            Ok(state) => match &**state {
                ResidencyState::Cold => (true, matches!(demand, ResidencyDemand::Full)),
                ResidencyState::Head { .. } => (false, matches!(demand, ResidencyDemand::Full)),
                ResidencyState::Full(_) | ResidencyState::Damaged { .. } => (false, false),
            },
            // If the cell gate is contended, do not speculate; `demand` checks
            // the locked state and re-runs `admit_working_set_for` outside the
            // lock if a cold load is actually needed.
            Err(_) => (false, false),
        };
        self.admit_working_set_for(need_head, need_full)
    }

    /// Reserve estimated working-set and in-flight fill bytes outside the cell
    /// gate for an exact `(need_head, need_full)` transition pair.
    pub(super) fn admit_working_set_for(
        &self,
        need_head: bool,
        need_full: bool,
    ) -> Option<WorkingSetAdmission> {
        let binding = self.working_set.get()?;
        let cold = self.backing.as_ref()?;
        let head_bytes = if need_head { cold.head.length } else { 0 };
        let full_bytes = if need_full {
            cold.head.length.saturating_add(cold.history.length)
        } else {
            0
        };
        let encoded = head_bytes.saturating_add(full_bytes);
        let mut admission = WorkingSetAdmission::default();
        if encoded == 0 {
            return Some(admission);
        }
        let estimate = binding
            .accountant
            .estimate(ResidencyLevelKind::Head, head_bytes)
            .saturating_add(
                binding
                    .accountant
                    .estimate(ResidencyLevelKind::Full, full_bytes),
            );
        let fill = head_bytes.max(full_bytes);
        match binding.accountant.reserve_fill(fill) {
            Ok(charge) => admission._fill = Some(charge),
            Err(refusal) => {
                admission.exhausted = Some(refusal);
                return Some(admission);
            }
        }
        match binding.accountant.reserve(binding.pool, estimate, self) {
            Ok(charge) => {
                admission.prepaid = Some(charge);
                admission.reserved_head = need_head;
                admission.reserved_full = need_full;
            }
            Err(refusal) => {
                // Drop in-flight fill budget immediately on pool refusal so we
                // do not hold fill capacity across a contended gate wait.
                admission._fill = None;
                admission.exhausted = Some(refusal);
            }
        }
        Some(admission)
    }

    /// A load that needs working-set bytes refuses before any I/O when the
    /// pre-lock admission found genuine saturation.
    pub(super) fn require_capacity(
        &self,
        admission: Option<&WorkingSetAdmission>,
    ) -> Result<(), Failure> {
        match admission {
            Some(admission) if admission.exhausted.is_some() => Err(self.exhausted()),
            _ => Ok(()),
        }
    }

    /// Prove the loaded image's basis and attach its measured charge before
    /// it can be cached or published. Growth beyond the prepaid estimate
    /// evicts other cells with `try_lock` only; this cell is never a victim.
    pub(super) fn charge_loaded(
        &self,
        region: Arc<ResidentRegion>,
        admission: Option<&mut WorkingSetAdmission>,
        level: ResidencyLevelKind,
        encoded: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        let (Some(admission), Some(binding)) = (admission, self.working_set.get()) else {
            return Ok(region);
        };
        let schema = self
            .backing
            .as_ref()
            .map(|cold| cold.schema_carrier_digest)
            .ok_or_else(|| self.failure(Cause::SourceUnavailable))?;
        // Rewarm after eviction never serves an image of another basis, even
        // when the shared source now names a different branch's bytes.
        if BasisIdentity::observed(self.partition_id, &region, schema, level)
            != self.expected_basis(level)
        {
            return Err(Failure::StorageUnavailable(
                self.failure(Cause::IntegrityMismatch),
            ));
        }
        let bytes = resident_region_bytes(&region).ok_or_else(|| self.exhausted())?;
        binding.accountant.observe_expansion(level, encoded, bytes);
        let charge = match admission
            .prepaid
            .as_mut()
            .and_then(|prepaid| prepaid.split_off(bytes))
        {
            Some(charge) => charge,
            None => {
                let mut held = admission.prepaid.take();
                let short = bytes - held.as_ref().map_or(0, WorkingSetCharge::bytes);
                let extra = binding
                    .accountant
                    .reserve(binding.pool, short, self)
                    .map_err(|_| self.exhausted())?;
                match held.as_mut() {
                    Some(held) => {
                        held.absorb(extra);
                    }
                    None => held = Some(extra),
                }
                held.expect("a charge was just reserved")
            }
        };
        region
            .working_set
            .set(charge)
            .map_err(|_| self.failure(Cause::SourceUnavailable))?;
        Ok(region)
    }
}

/// Measured bytes of one resident image: its allocation plus every
/// authoritative, diagnostic, retention, commitment, index and cache allocation
/// it owns.
fn resident_region_bytes(region: &ResidentRegion) -> Option<u64> {
    let inventory = region.partition.allocation_inventory().ok()?;
    let commitment_bytes = region
        .commitment
        .as_ref()
        .map_or(0, |commitment| commitment.allocation_bytes());
    let kind_index_bytes = region
        .kind_indexes
        .as_ref()
        .map_or(0, |indexes| indexes.allocation_bytes());
    [
        inventory.authoritative_bytes,
        inventory.diagnostic_bytes,
        inventory.retention_metadata_bytes,
        inventory.allocator_bookkeeping_bytes,
        inventory.optional_cache_bytes,
        commitment_bytes,
        kind_index_bytes,
        std::mem::size_of::<ResidentRegion>() as u64,
    ]
    .into_iter()
    .try_fold(0_u64, u64::checked_add)
}
