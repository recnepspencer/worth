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

/// What resuming the successor of a dependent's upstream row left the wave.
pub(in crate::domain_computation::primary_graph) enum HeldUpstream {
    /// The successor finished with the row's Ready.
    Ready(SelectedReadyReadmission),
    /// The successor deferred, or none was held.
    Unfinished,
    /// The World superseded the successor before it published. The Ready it
    /// replaced answers for the occurrence again; certify the row once more.
    GaveBack,
}

pub(in crate::domain_computation::primary_graph) enum ContinuationCustody<
    'a,
    Schema: ApplicationSchema,
> {
    Caller(&'a mut RequiredContinuations<Schema>, &'a mut usize),
    Queue(&'a mut RequiredContinuations<Schema>),
}

/// Resume the successor that refreshes `key`'s row: from `custody`, or from
/// the registry, where a queue frame left it. A finished successor is
/// `custody`'s newest entry: the predecessor edge its dependents still name
/// resolves through it. One whose source moved before it published is
/// dropped, and its occurrence goes back to the Ready it replaced.
pub(in crate::domain_computation::primary_graph) fn resume_held_upstream<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    custody: ContinuationCustody<'_, Schema>,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<HeldUpstream, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let (custody, caller_contacts) = match custody {
        ContinuationCustody::Caller(custody, contacts) => (custody, Some(contacts)),
        ContinuationCustody::Queue(custody) => (custody, None),
    };
    if let Some(index) = custody.position_of(key, admission)? {
        let result = custody.entries[index].resume(runtime, principal, request, branch, admission);
        custody.entries[index].report_caller_contacts(caller_contacts);
        let finished = matches!(result, Ok(Some(_)));
        if finished || result.is_err() {
            let moved = (custody.entries.len() - index)
                .checked_mul(std::mem::size_of::<RequiredFreshProgress<Schema>>())
                .and_then(|work| u64::try_from(work).ok())
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(moved)
                .map_err(|_| work_denial())?;
        }
        if finished {
            custody.entries[index..].rotate_left(1);
        }
        return match result {
            Ok(Some(ready)) => Ok(HeldUpstream::Ready(ready)),
            Ok(None) => Ok(HeldUpstream::Unfinished),
            Err(stop)
                if superseded(&stop)
                    && runtime
                        .output_demands
                        .give_back_replaced_ready(key, admission)? =>
            {
                drop(custody.entries.remove(index));
                Ok(HeldUpstream::GaveBack)
            }
            Err(stop) => Err(stop),
        };
    }
    let Some(held) = runtime.output_demands.take_held_successor(key, admission)? else {
        return Ok(HeldUpstream::Unfinished);
    };
    let mut held = held
        .downcast::<HeldFrameSuccessor<Schema>>()
        .unwrap_or_else(|_| unreachable!("the registry holds only this runtime's successors"));
    let result = held
        .progress
        .resume(runtime, principal, request, branch, admission);
    held.progress.report_caller_contacts(None);
    let ready = match result {
        Ok(Some(ready)) => ready,
        Err(stop)
            if superseded(&stop)
                && runtime
                    .output_demands
                    .give_back_replaced_ready(key, admission)? =>
        {
            drop(stop);
            drop(held);
            return Ok(HeldUpstream::GaveBack);
        }
        // Refused custody, the successor ends instead of holding its own.
        Err(stop) if super::progress::refused_custody(&stop) => {
            drop(held);
            return Err(stop);
        }
        unfinished => {
            // An unfinished successor returns to its row, which drops it if
            // the row has moved on meanwhile.
            drop(
                runtime
                    .output_demands
                    .hold_required_successor(held, held_key::<Schema>),
            );
            return unfinished.map(|_| HeldUpstream::Unfinished);
        }
    };
    let HeldFrameSuccessor { progress, .. } = *held;
    let slot = custody.prepare_slot(&runtime.output_demands, admission)?;
    admission
        .charge_external_work(slot.installation_work())
        .map_err(|_| work_denial())?;
    slot.push(progress);
    Ok(HeldUpstream::Ready(ready))
}

/// Only the World superseding the successor gives its occurrence back; any
/// other stop is the wave's to report.
fn superseded(stop: &WorthQueryOutputDemandDenial) -> bool {
    stop.kind() == super::super::WorthQueryOutputDemandDenialKind::Superseded
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
