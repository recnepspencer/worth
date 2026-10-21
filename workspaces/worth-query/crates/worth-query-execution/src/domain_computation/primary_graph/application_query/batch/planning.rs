//! Exact installed query slots for a finite caller-described schedule.
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryBinding,
    WorthQueryInstalledApplicationQueryIdentity,
};

use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// An owner-issued schedule for an installed binding and actual
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

    query: WorthQueryInstalledApplicationQueryIdentity,
    count: usize,

    roots: usize,
}

pub(super) struct InstalledBatchPlan {
    runtime: u64,
    schema: ApplicationSchemaBindingIdentity,

    slots: BTreeMap<WorthQueryInstalledApplicationQueryIdentity, PlannedSlots>,
    pub(super) items: usize,

    pub(super) frozen: bool,
    pub(super) started: bool,
}

#[derive(Clone)]
struct PlannedSlots {
    remaining: usize,

    roots: usize,
}

pub(in crate::domain_computation::primary_graph) struct PlannedBatchItem {
    plan: Arc<Mutex<InstalledBatchPlan>>,
}

impl PlannedBatchItem {
    pub(in crate::domain_computation::primary_graph::application_query) fn belongs_to(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> bool {
        Arc::ptr_eq(&self.plan, &batch.plan)
    }
}

impl<Schema: worth_query_declaration::facade::application_schema::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Registers a finite schedule using genuine installed bindings.
    /// Zero reads stays zero. The count is scheduling data, not membership proof.
    pub fn plan_application_query_batch_reads<B: ApplicationQueryBinding<Schema>>(
        &self,
        binding: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
        count: usize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        let limits = self.resolve_application_query_limits(binding.limits());
        self.plan_installed_query_batch_reads(binding.query(), count, limits.maximum_results())
    }

    /// Registers native installed reads for callers already owning their query
    /// handle. This describes slots only: each read still needs fresh admission.
    pub fn plan_installed_query_batch_reads<Q, P, R, S>(
        &self,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
            Schema,
            Q,
            P,
            R,
            S,
        >,
        count: usize,
        maximum_results: std::num::NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        self.installed_schema
            .validate_installed_query(query)
            .map_err(|_| WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan)?;
        if let Some(maximum) =
            std::num::NonZeroUsize::new(query.read_graph().maximum_result_count())
            && maximum_results > maximum
        {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ItemLimit {
                required: maximum_results.get(),
                maximum: maximum.get(),
            });
        }
        Ok(WorthQueryApplicationQueryBatchReadPlan {
            runtime: self.runtime.authority_identity().as_u64(),
            schema: self.installed_schema.binding_identity().clone(),
            query: query.identity().clone(),
            count,
            roots: maximum_results.get(),
        })
    }

    /// Opens one installed schedule. A census may run before its sole
    /// extension; otherwise call `freeze_planned_reads` before the first read.
    pub fn application_query_batch_from_installed_reads(
        &self,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<
        WorthQueryApplicationQueryBatchAdmission,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        let mut plan = InstalledBatchPlan {
            runtime: self.runtime.authority_identity().as_u64(),
            schema: self.installed_schema.binding_identity().clone(),

            slots: BTreeMap::new(),
            items: 0,

            frozen: false,
            started: false,
        };
        plan.extend(reads, false)?;
        Ok(WorthQueryApplicationQueryBatchAdmission {
            plan: Arc::new(Mutex::new(plan)),
        })
    }

    /// Extends once from discovered planning data and freezes atomically.
    /// Already consumed slots and original native custody remain unchanged.
    pub fn extend_application_query_batch_once(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        let plan = &batch.plan;
        let mut plan = plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if plan.runtime != self.runtime.authority_identity().as_u64()
            || plan.schema != self.installed_schema.binding_identity()
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
        let mut items = self.items;
        for read in reads {
            if read.runtime != self.runtime || read.schema != self.schema {
                return Err(ForeignPlan);
            }
            if let Some(old) = self.slots.get(&read.query)
                && old.roots != read.roots
            {
                return Err(ForeignPlan);
            }
            items = items.checked_add(read.count).ok_or(CounterOverflow)?;
        }
        let mut slots = self.slots.clone();
        for read in reads {
            if read.count == 0 {
                continue;
            }
            let slot = slots.entry(read.query.clone()).or_insert(PlannedSlots {
                remaining: 0,

                roots: read.roots,
            });
            slot.remaining = slot
                .remaining
                .checked_add(read.count)
                .ok_or(CounterOverflow)?;
        }
        self.slots = slots;
        self.items = items;
        self.frozen = freeze;
        Ok(())
    }
}

impl WorthQueryApplicationQueryBatchAdmission {
    /// Closes an initial affected-only schedule, including an empty schedule.
    pub fn freeze_planned_reads(
        &self,
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        {
            let plan = &self.plan;
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
        maximum_roots: usize,
    ) -> Result<Option<PlannedBatchItem>, WorthQueryApplicationQueryBatchResourceDenial> {
        let plan = &self.plan;
        let mut plan = plan
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if plan.runtime != runtime || &plan.schema != schema {
            return Err(WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan);
        }
        plan.started = true;
        let slot = plan
            .slots
            .get_mut(query)
            .ok_or(WorthQueryApplicationQueryBatchResourceDenial::UnplannedRead)?;
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
            plan: Arc::clone(&self.plan),
        }))
    }
}

#[cfg(test)]
mod tests;
