//! Resume an unfinished successor, and hand a queue frame's unfinished
//! successors to registry custody on their own rows.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    HeldRequiredSuccessor, WorthQueryOutputDemandKey,
};

/// The registry's copy of one frame successor and the custody it is funded by.
struct HeldFrameSuccessor<Schema>
where
    Schema: ApplicationSchema,
{
    progress: RequiredFreshProgress<Schema>,
    // Declared after `progress`: the ticket refunds after its storage frees.
    _capacity: RequiredOutputCustodyCapacity,
}

fn held_key<Schema>(held: &HeldRequiredSuccessor) -> &WorthQueryOutputDemandKey
where
    Schema: ApplicationSchema + 'static,
{
    held.downcast_ref::<HeldFrameSuccessor<Schema>>()
        .expect("the registry holds only this runtime's successors")
        .progress
        .interest()
        .key()
}

impl<Schema> RequiredContinuations<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The entry of the successor that refreshes `key`'s row, if any.
    fn position_of(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<usize>, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(u64::try_from(self.entries.len()).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        Ok(self
            .entries
            .iter()
            .rposition(|entry| entry.interest().key() == key))
    }

    /// A queue frame ends with its wave. Each successor moves to registry
    /// custody on its own row, so a later advance can finish that row; the
    /// registry drops one whose row is already Ready, which outlives it for
    /// the owners that rejoin it.
    pub(in crate::domain_computation::primary_graph) fn hold_unfinished(
        mut self,
        registry: &WorthQueryOutputDemandRegistry,
    ) {
        for progress in std::mem::take(&mut self.entries) {
            let bytes = std::mem::size_of::<HeldFrameSuccessor<Schema>>();
            // Without custody the successor ends here, as it would have.
            let Ok(capacity) = registry.reserve_held_successor_capacity(bytes) else {
                continue;
            };
            let held: HeldRequiredSuccessor = Box::new(HeldFrameSuccessor {
                progress,
                _capacity: capacity,
            });
            drop(registry.hold_required_successor(held, held_key::<Schema>));
        }
    }
}

/// Resume the successor that refreshes `key`'s row: from `custody`, or from
/// the registry, where a queue frame left it. Returns the row's Ready once
/// it completes; a deferred or absent successor returns `None`. A finished
/// successor is `custody`'s newest entry: the predecessor edge its
/// dependents still name resolves through it.
pub(in crate::domain_computation::primary_graph) fn resume_held_upstream<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    custody: &mut RequiredContinuations<Schema>,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    if let Some(index) = custody.position_of(key, admission)? {
        let result = custody.entries[index].resume(runtime, principal, request, branch, admission);
        if matches!(result, Ok(Some(_))) {
            let moved = (custody.entries.len() - index)
                .checked_mul(std::mem::size_of::<RequiredFreshProgress<Schema>>())
                .and_then(|work| u64::try_from(work).ok())
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(moved)
                .map_err(|_| work_denial())?;
            custody.entries[index..].rotate_left(1);
        }
        return result;
    }
    let Some(held) = runtime.output_demands.take_held_successor(key, admission)? else {
        return Ok(None);
    };
    let mut held = held
        .downcast::<HeldFrameSuccessor<Schema>>()
        .unwrap_or_else(|_| unreachable!("the registry holds only this runtime's successors"));
    let result = held
        .progress
        .resume(runtime, principal, request, branch, admission);
    let Ok(Some(ready)) = result else {
        // An unfinished successor returns to its row, which drops it if the
        // row has moved on meanwhile.
        drop(
            runtime
                .output_demands
                .hold_required_successor(held, held_key::<Schema>),
        );
        return result;
    };
    let HeldFrameSuccessor { progress, .. } = *held;
    let slot = custody.prepare_slot(&runtime.output_demands, admission)?;
    admission
        .charge_external_work(slot.installation_work())
        .map_err(|_| work_denial())?;
    slot.push(progress);
    Ok(Some(ready))
}

impl<Schema> RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Run the successor's row again and drive its checkpoint to Ready. A
    /// row already Ready is not run again.
    fn resume(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        let registry = &runtime.output_demands;
        if registry
            .interest_ready_readmission(self.interest(), admission)?
            .is_some()
        {
            return Ok(None);
        }
        self.successor
            .resume(runtime, principal, request, branch, admission)?;
        loop {
            if let Some(ready) = registry.interest_ready_readmission(self.interest(), admission)? {
                return Ok(Some(ready));
            }
            if !self.advance_checkpoint(runtime, request, admission)? {
                return Ok(None);
            }
        }
    }
}

impl<Schema, Family> TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    /// Advance the typed demand from its frozen source under the commit
    /// authority that issued it.
    pub(super) fn run_again(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        let demand = self
            .demand
            .as_mut()
            .expect("installed successor retains its typed demand");
        admission
            .charge_external_work(
                u64::try_from(std::mem::size_of::<WorthQueryProducerCommitAuthority>())
                    .map_err(|_| work_denial())?,
            )
            .map_err(|_| work_denial())?;
        let authority = demand
            .progression_provenance
            .required()
            .expect("installed required successor retains its executed mode")
            .commit_authority()
            .clone();
        runtime.advance_retained_with_commit_authority(
            demand, principal, request, branch, authority, admission,
        )
    }
}
