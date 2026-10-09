//! Caller-owned custody for unfinished required-output successors.
//!
//! A fresh required admission can replace a Ready interest before it reaches
//! Ready itself. Its typed demand, not just its key, must survive a Pending or
//! post-admission refusal so a later request can resume the real interest. A
//! stop that returned its row to the Ready it reopened ends it instead: the
//! next wave claims that refresh again, after the entry frees its custody.

use std::any::{Any, TypeId};

use worth_query_installation::facade::ApplicationSchema;

use super::{
    super::WorthQueryProducerCommitAuthority,
    required_provenance::{DemandProgressionProvenance, RequiredSuccessorProvenance},
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryProducerOutputFamily,
};
mod held;
mod prepared;
mod progress;
mod promotion;
mod unavailable;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::registry::InstalledProducerProvider,
    application_output_demand::{
        RequestedOutputReadClaims, RequiredOutputCustodyCapacity, SelectedReadyReadmission,
        SelectedRequiredRefreshClaim, WorthQueryOutputDemandInterest,
        WorthQueryOutputDemandRegistry,
    },
    output_lineage::invalidation::InvalidationEditAdmission,
    WorthQueryApplicationProjection, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};

pub(in crate::domain_computation::primary_graph) use held::{resume_held_upstream, HeldUpstream};

pub(in crate::domain_computation::primary_graph) enum RequiredFreshOutcome {
    Advanced,
    Refused(WorthQueryOutputDemandDenial),
}

/// This trait is private to the admitted-demand owner. The erased value is
/// always an actual typed admitted demand, with its original real interest.
trait ErasedRequiredSuccessor<Schema: ApplicationSchema>: Send + Sync {
    fn interest(&self) -> &WorthQueryOutputDemandInterest;
    fn producer_identity(&self) -> &str;
    fn family_type(&self) -> TypeId;
    fn concrete_type(&self) -> TypeId;
    fn progression(&self) -> &RequiredSuccessorProvenance;
    fn validate_for_current_handoff(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial>;
    fn predecessor(&self) -> &SelectedReadyReadmission;
    fn producer_contacts(&self) -> usize;
    fn continuations_empty(&self) -> bool;
    fn retain_requested(&mut self, claims: &mut RequestedOutputReadClaims);
    /// Move to the newest row of the occurrence once this one is superseded.
    fn follow_refresh(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial>;
    fn advance_checkpoint(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial>;
    /// Run the successor's own row again under the authority that issued it.
    fn resume(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

struct TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    demand: Option<WorthQueryAdmittedOutputDemand<Schema, Family>>,
    pending_claim: Option<SelectedRequiredRefreshClaim>,
    pending_provenance: Option<RequiredSuccessorProvenance>,
    predecessor: Option<SelectedReadyReadmission>,
}

impl<Schema, Family> ErasedRequiredSuccessor<Schema> for TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema> + 'static,
    WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    fn interest(&self) -> &WorthQueryOutputDemandInterest {
        self.demand
            .as_ref()
            .and_then(|demand| demand.interest.as_ref())
            .expect("installed required successor retains its real interest")
    }

    fn producer_identity(&self) -> &str {
        &self
            .demand
            .as_ref()
            .expect("installed required successor retains its typed demand")
            .selected
            .identity
    }

    fn family_type(&self) -> TypeId {
        TypeId::of::<Family>()
    }

    fn concrete_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn progression(&self) -> &RequiredSuccessorProvenance {
        self.demand
            .as_ref()
            .and_then(|demand| demand.progression_provenance.required())
            .expect("installed required successor retains its executed mode")
    }

    fn predecessor(&self) -> &SelectedReadyReadmission {
        self.predecessor
            .as_ref()
            .expect("installed required successor retains its exact predecessor Ready")
    }

    fn validate_for_current_handoff(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let demand = self
            .demand
            .as_ref()
            .expect("installed successor owns its demand");
        self.progression().validate_for_execution(
            self.progression().commit_authority(),
            &demand.installed_entry.edition,
            admission,
        )
    }

    fn producer_contacts(&self) -> usize {
        self.demand
            .as_ref()
            .expect("installed required successor retains its typed demand")
            .producer_contacts_in_this_demand
    }

    fn continuations_empty(&self) -> bool {
        self.demand.as_ref().is_some_and(|demand| {
            demand.required_continuations.entries.is_empty()
                && demand.required_continuations.capacity.is_none()
                && demand.required_continuations.requested.is_empty()
        })
    }

    fn retain_requested(&mut self, claims: &mut RequestedOutputReadClaims) {
        self.demand
            .as_mut()
            .expect("installed successor owns its demand")
            .required_continuations
            .requested
            .absorb(claims);
    }

    fn follow_refresh(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.demand
            .as_mut()
            .expect("installed required successor retains its typed demand")
            .rejoin_refreshed_output(registry, admission)
    }

    fn advance_checkpoint(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let demand = self
            .demand
            .as_mut()
            .expect("installed successor retains its typed demand");
        if demand.unpublished_selected_checkpoint.is_some() {
            return Ok(false);
        }
        let interest = demand
            .interest
            .as_ref()
            .expect("installed successor retains its real Interest");
        let prepared = runtime
            .output_demands
            .prepare_selected_checkpoint_finish(interest, admission)?;
        let Some((finish, claim, checkpoint)) = prepared.claim()? else {
            return Ok(false);
        };
        runtime.advance_selected_output_checkpoint(
            &demand.selected.identity,
            claim,
            checkpoint,
            request,
            demand.resources,
            &mut demand.unpublished_selected_checkpoint,
            finish,
            admission,
        )
    }

    fn resume(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        self.run_again(runtime, principal, request, branch, admission)
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// The allocation is made and funded before successor admission or effects.
/// The slot stays empty until the registry has issued the actual typed demand.
pub(in crate::domain_computation::primary_graph) struct PreparedRequiredFreshSlot<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    successor: Box<TypedRequiredSuccessor<Schema, Family>>,
    // Declared after the Box: the final ticket refunds after Box storage frees.
    capacity: RequiredOutputCustodyCapacity,
}

/// Move-only post-admission result. The caller installs this into its funded
/// continuation slot before reading the outcome or returning Pending/Err.
pub(in crate::domain_computation::primary_graph) struct RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema,
{
    outcome: Option<RequiredFreshOutcome>,
    successor: Box<dyn ErasedRequiredSuccessor<Schema>>,
    // This field must drop after `successor`; the Box owns physical backing.
    capacity: RequiredOutputCustodyCapacity,
}

/// One caller's actual unresolved dependent work. The Vec backing has its
/// own final-owner ticket; each entry separately funds its erased Box.
pub(super) struct RequiredContinuations<Schema>
where
    Schema: ApplicationSchema,
{
    entries: Vec<RequiredFreshProgress<Schema>>,
    pub(super) requested: RequestedOutputReadClaims,
    capacity: Option<RequiredOutputCustodyCapacity>,
}

impl<Schema> Default for RequiredContinuations<Schema>
where
    Schema: ApplicationSchema,
{
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            requested: Default::default(),
            capacity: None,
        }
    }
}

/// Move-only storage prepared before any successor registration or effect.
/// Installing one Progress afterward has no fallible allocation or admission.
pub(super) struct PreparedRequiredContinuationSlot<'caller, Schema>
where
    Schema: ApplicationSchema,
{
    caller: &'caller mut RequiredContinuations<Schema>,
    replacement: Option<Vec<RequiredFreshProgress<Schema>>>,
    capacity: Option<RequiredOutputCustodyCapacity>,
    installation_work: u64,
}

impl<Schema> RequiredContinuations<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The last installed successor is the exact Fresh result of the current
    /// wave step. The caller may borrow its issued Interest after the slot's
    /// infallible install, while this container keeps typed custody alive.
    pub(super) fn last(&self) -> Option<&RequiredFreshProgress<Schema>> {
        self.entries.last()
    }

    pub(super) fn last_mut(&mut self) -> Option<&mut RequiredFreshProgress<Schema>> {
        self.entries.last_mut()
    }

    pub(super) fn prepare_slot(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedRequiredContinuationSlot<'_, Schema>, WorthQueryOutputDemandDenial> {
        // Ended refreshes free their custody before this slot reserves any.
        self.end_restored(registry, admission)?;
        let old_len = self.entries.len();
        // Installation compares the new successor with each held entry.
        admission
            .charge_external_work(u64::try_from(old_len).map_err(|_| work_denial())? + 3)
            .map_err(|_| work_denial())?;
        let item_bytes = std::mem::size_of::<RequiredFreshProgress<Schema>>();
        let next_len = old_len.checked_add(1).ok_or_else(capacity_denial)?;
        let relocation_work = if old_len < self.entries.capacity() {
            0
        } else {
            old_len.checked_mul(item_bytes).ok_or_else(work_denial)?
        };
        let installation_work = u64::try_from(
            relocation_work
                .checked_add(item_bytes)
                .ok_or_else(work_denial)?,
        )
        .map_err(|_| work_denial())?;
        if old_len < self.entries.capacity() {
            return Ok(PreparedRequiredContinuationSlot {
                caller: self,
                replacement: None,
                capacity: None,
                installation_work,
            });
        }
        let old_bytes = self
            .entries
            .capacity()
            .checked_mul(item_bytes)
            .ok_or_else(capacity_denial)?;
        let new_bytes = next_len
            .checked_mul(item_bytes)
            .ok_or_else(capacity_denial)?;
        let peak_bytes = old_bytes
            .checked_add(new_bytes)
            .ok_or_else(capacity_denial)?;
        let capacity =
            registry.reserve_required_continuation_capacity(new_bytes, peak_bytes, admission)?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(next_len)
            .map_err(|_| capacity_denial())?;
        if replacement.capacity() != next_len {
            return Err(capacity_denial());
        }
        Ok(PreparedRequiredContinuationSlot {
            caller: self,
            replacement: Some(replacement),
            capacity: Some(capacity),
            installation_work,
        })
    }

    pub(super) fn take_all(&mut self) -> Self {
        std::mem::take(self)
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        super::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "required continuation exceeds carried request work",
    )
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required continuation exceeds retained capacity",
    )
}
