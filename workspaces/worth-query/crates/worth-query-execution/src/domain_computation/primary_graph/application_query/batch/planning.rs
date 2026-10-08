//! Installed resource policy for one finite, caller-described read schedule.
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryBinding,
    WorthQueryInstalledApplicationQueryIdentity,
};

use super::*;
use crate::domain_computation::execution_runtime::WorthQueryApplicationQueryResourceProfile;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// An owner-produced resource description for an installed binding and actual
/// planned read count. It grants no scope, authorization, or currentness.
///
/// Callers cannot change the installed issuer or scheduled count.
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryBatchReadPlan;
/// fn forge(mut plan: WorthQueryApplicationQueryBatchReadPlan) {
///     plan.count = 100;
/// }
/// ```
pub struct WorthQueryApplicationQueryBatchReadPlan {
    runtime: u64,
    schema: ApplicationSchemaBindingIdentity,
    profile: WorthQueryApplicationQueryResourceProfile,
    query: WorthQueryInstalledApplicationQueryIdentity,
    binding: &'static str,
    count: usize,
    work: usize,
    roots: usize,
    result: usize,
}

pub(super) struct InstalledBatchPlan {
    runtime: u64,
    schema: ApplicationSchemaBindingIdentity,
    profile: WorthQueryApplicationQueryResourceProfile,
    slots: BTreeMap<WorthQueryInstalledApplicationQueryIdentity, PlannedSlots>,
    pub(super) items: usize,
    pub(super) work: usize,
    pub(super) bytes: usize,
    pub(super) result: NonZeroUsize,
    frozen: bool,
}

#[derive(Clone)]
struct PlannedSlots {
    binding: &'static str,
    remaining: usize,
    work: usize,
    roots: usize,
}

pub(in crate::domain_computation::primary_graph) struct PlannedBatchItem {
    totals: Arc<Mutex<BatchTotals>>,
}

impl PlannedBatchItem {
    pub(in crate::domain_computation::primary_graph::application_query) fn belongs_to(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> bool {
        Arc::ptr_eq(&self.totals, &batch.totals)
    }
}

impl<Schema: worth_query_declaration::facade::application_schema::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Describes configured resources without exposing execution authority.
    pub fn application_query_resource_profile(&self) -> WorthQueryApplicationQueryResourceProfile {
        self.runtime.application_query_resource_profile()
    }

    /// Quotes a finite schedule using the genuine installed binding ceilings.
    /// Zero reads stays zero. The count is scheduling data, not membership proof.
    pub fn plan_application_query_batch_reads<B: ApplicationQueryBinding<Schema>>(
        &self,
        binding: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
        count: usize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        self.installed_schema
            .validate_installed_query(binding.query())
            .map_err(|_| WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan)?;
        let profile = self.application_query_resource_profile();
        let limits = self.resolve_application_query_limits(binding.limits());
        let result = profile
            .maximum_result_bytes_per_root()
            .get()
            .checked_mul(limits.maximum_results().get())
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        Ok(WorthQueryApplicationQueryBatchReadPlan {
            runtime: self.runtime.authority_identity().as_u64(),
            schema: self.installed_schema.binding_identity().clone(),
            profile,
            query: binding.query().identity().clone(),
            binding: B::IDENTITY,
            count,
            work: limits.maximum_work().get(),
            roots: limits.maximum_results().get(),
            result,
        })
    }

    /// Starts one installed-profile loan. A census may run before its sole
    /// extension; otherwise call `freeze_planned_reads` before the first read.
    pub fn application_query_batch_from_installed_reads(
        &self,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<
        WorthQueryApplicationQueryBatchAdmission,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        let profile = self.application_query_resource_profile();
        let mut plan = InstalledBatchPlan {
            runtime: self.runtime.authority_identity().as_u64(),
            schema: self.installed_schema.binding_identity().clone(),
            profile,
            slots: BTreeMap::new(),
            items: 0,
            work: 0,
            bytes: 0,
            result: profile.maximum_result_bytes_per_root(),
            frozen: false,
        };
        plan.extend(reads, false)?;
        Ok(WorthQueryApplicationQueryBatchAdmission {
            policy: BatchPolicy::Installed(Arc::new(Mutex::new(plan))),
            totals: Arc::new(Mutex::new(BatchTotals::default())),
        })
    }

    /// Extends once from discovered planning data and freezes atomically.
    /// Spent census work and every original custody claim remain unchanged.
    pub fn extend_application_query_batch_once(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let BatchPolicy::Installed(plan) = &batch.policy else {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan);
        };
        let mut plan = plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if plan.runtime != self.runtime.authority_identity().as_u64()
            || plan.schema != self.installed_schema.binding_identity()
            || plan.profile != self.application_query_resource_profile()
        {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan);
        }
        plan.extend(reads, true)
    }
}

impl InstalledBatchPlan {
    fn extend(
        &mut self,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
        freeze: bool,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        use WorthQueryApplicationQueryBatchResourceDenial::{
            CounterOverflow, ForeignPlan, PlanningFrozen,
        };
        if self.frozen {
            return Err(PlanningFrozen);
        }
        let (mut items, mut work, mut bytes, mut result) =
            (self.items, self.work, self.bytes, self.result);
        for read in reads {
            if read.runtime != self.runtime
                || read.schema != self.schema
                || read.profile != self.profile
            {
                return Err(ForeignPlan);
            }
            if let Some(old) = self.slots.get(&read.query) {
                if old.binding != read.binding {
                    return Err(ForeignPlan);
                }
            }
            items = items.checked_add(read.count).ok_or(CounterOverflow)?;
            work = work
                .checked_add(read.work.checked_mul(read.count).ok_or(CounterOverflow)?)
                .ok_or(CounterOverflow)?;
            // Result staging and the live kernel buffer may coexist. Source and
            // basis metadata are quoted by their real owners before each copy.
            bytes = bytes
                .checked_add(
                    read.result
                        .checked_mul(read.count)
                        .and_then(|n| n.checked_mul(2))
                        .ok_or(CounterOverflow)?,
                )
                .ok_or(CounterOverflow)?;
            if read.count != 0 {
                result = result.max(NonZeroUsize::new(read.result).ok_or(CounterOverflow)?);
            }
        }
        let mut slots = self.slots.clone();
        for read in reads {
            if read.count == 0 {
                continue;
            }
            let slot = slots.entry(read.query.clone()).or_insert(PlannedSlots {
                binding: read.binding,
                remaining: 0,
                work: read.work,
                roots: read.roots,
            });
            slot.remaining = slot
                .remaining
                .checked_add(read.count)
                .ok_or(CounterOverflow)?;
        }
        self.slots = slots;
        (self.items, self.work, self.bytes, self.result) = (items, work, bytes, result);
        self.frozen = freeze;
        Ok(())
    }
}

impl WorthQueryApplicationQueryBatchAdmission {
    /// Closes an initial affected-only schedule, including an empty schedule.
    pub fn freeze_planned_reads(
        &self,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        if let BatchPolicy::Installed(plan) = &self.policy {
            let mut plan = plan
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if plan.frozen {
                return Err(WorthQueryApplicationQueryBatchResourceDenial::PlanningFrozen);
            }
            plan.frozen = true;
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn take_planned_read(
        &self,
        runtime: u64,
        schema: &ApplicationSchemaBindingIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        maximum_work: usize,
        maximum_roots: usize,
    ) -> Result<Option<PlannedBatchItem>, WorthQueryApplicationQueryBatchResourceDenial> {
        let BatchPolicy::Installed(plan) = &self.policy else {
            return Ok(None);
        };
        let mut plan = plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if plan.runtime != runtime || &plan.schema != schema {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan);
        }
        let slot = plan
            .slots
            .get_mut(query)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::UnplannedRead)?;
        if maximum_work > slot.work {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::WorkLimit {
                required: maximum_work,
                maximum: slot.work,
            });
        }
        if maximum_roots > slot.roots {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ItemLimit {
                required: maximum_roots,
                maximum: slot.roots,
            });
        }
        slot.remaining = slot
            .remaining
            .checked_sub(1)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::UnplannedRead)?;
        Ok(Some(PlannedBatchItem {
            totals: Arc::clone(&self.totals),
        }))
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn claim_source_memory(
        &self,
        bytes: usize,
    ) -> Result<WorthQueryApplicationQueryBatchMemory, WorthQueryApplicationQueryBatchResourceDenial>
    {
        if let BatchPolicy::Installed(plan) = &self.policy {
            let mut plan = plan
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plan.bytes = plan
                .bytes
                .checked_add(bytes)
                .ok_or(WorthQueryApplicationQueryBatchResourceDenial::CounterOverflow)?;
        }
        self.claim_memory(bytes)
    }
}
