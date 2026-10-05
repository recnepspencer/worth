//! Extract only the actual typed successor after the caller's exact join.

use super::super::WorthQueryOutputDemandDenialKind;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;

const FAMILY_SUBJECT: &str = "required successor family differs from caller demand";

impl<Schema> RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// A denial returns the same move-only Progress and its live Interest.
    /// The caller completes its Ready and registry joins before invoking this.
    pub(super) fn into_typed_admitted<Family>(
        self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, (Self, WorthQueryOutputDemandDenial)>
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
        WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    {
        if let Err(stop) = prepare_typed_extraction::<Schema, Family>(&self, admission) {
            return Err((self, stop));
        }
        let Self {
            outcome,
            successor,
            capacity,
        } = self;
        debug_assert!(outcome.is_none(), "only an installed progress is promoted");
        // This trait has one private concrete implementer. The preceding
        // concrete TypeId check makes this downcast infallible after custody
        // has been moved out of Progress.
        let mut typed = match successor
            .into_any()
            .downcast::<TypedRequiredSuccessor<Schema, Family>>()
        {
            Ok(typed) => typed,
            Err(_) => unreachable!("checked private required successor concrete type"),
        };
        let demand = typed
            .demand
            .take()
            .expect("installed required successor retains its demand");
        drop(typed);
        drop(capacity);
        Ok(demand)
    }
}

impl<Schema> RequiredContinuations<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Promote only the last installed successor of this caller's exact
    /// predecessor Ready, or of a successor on the caller's own refresh line
    /// of this wave (`continues_caller`). Every fallible authority check
    /// precedes the pop; a refused typed extraction restores the same
    /// move-only Progress.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn promote_caller_successor<
        Family,
    >(
        &mut self,
        registry: &WorthQueryOutputDemandRegistry,
        caller_ready: &SelectedReadyReadmission,
        selected_ready: &SelectedReadyReadmission,
        continues_caller: bool,
        supplied_mode: &WorthQueryProducerCommitAuthority,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<WorthQueryAdmittedOutputDemand<Schema, Family>>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
        WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    {
        admission
            .charge_external_work(6)
            .map_err(|_| empty_work())?;
        let Some(progress) = self.entries.last() else {
            return Ok(None);
        };
        if !progress.is_family::<Family>() {
            return Ok(None);
        }
        // A caller successor that cannot be certified is refreshed again on
        // this wave, and the newer refresh ends the older one's custody. Its
        // predecessor is then that earlier successor, which the wave reached
        // only through the caller's exact Ready.
        if !continues_caller
            && !progress
                .predecessor()
                .same_ready_cell(caller_ready, admission)?
        {
            return Ok(None);
        }
        if !registry.required_successor_is_live(caller_ready, progress.interest(), admission)? {
            return Ok(None);
        }
        if !selected_ready.matches_interest(progress.interest(), admission)? {
            return Ok(None);
        }
        progress
            .successor
            .validate_for_promotion(supplied_mode, admission)?;
        admission
            .charge_external_work(1)
            .map_err(|_| empty_work())?;
        if !progress.successor.continuations_empty() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "",
            ));
        }
        // This path will move the caller's continuation owner into the typed
        // successor and replace the caller demand. Clean Current without a
        // successor never reserves these carrier moves.
        let transfer_work = std::mem::size_of::<WorthQueryAdmittedOutputDemand<Schema, Family>>()
            .checked_mul(2)
            .and_then(|work| {
                work.checked_add(
                    std::mem::size_of::<RequiredContinuations<Schema>>().checked_mul(2)?,
                )
            })
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(empty_work)?;
        admission
            .charge_external_work(transfer_work)
            .map_err(|_| empty_work())?;
        admission
            .charge_external_work(
                std::mem::size_of::<RequiredFreshProgress<Schema>>()
                    .checked_mul(2)
                    .and_then(|work| u64::try_from(work).ok())
                    .ok_or_else(empty_work)?,
            )
            .map_err(|_| empty_work())?;
        let progress = self
            .entries
            .pop()
            .expect("checked last installed required successor");
        let typed = match progress.into_typed_admitted::<Family>(admission) {
            Ok(typed) => typed,
            Err((progress, denial)) => {
                self.entries.push(progress);
                return Err(denial);
            }
        };
        debug_assert!(typed.required_continuations.entries.is_empty());
        debug_assert!(typed.required_continuations.capacity.is_none());
        Ok(Some(typed))
    }
}

fn prepare_typed_extraction<Schema, Family>(
    progress: &RequiredFreshProgress<Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema> + 'static,
{
    admission
        .charge_external_work(1)
        .map_err(|_| empty_work())?;
    let backing = FAMILY_SUBJECT
        .len()
        .checked_add(std::mem::size_of::<String>())
        .ok_or_else(empty_capacity)?;
    admission
        .admit_read_scratch(u64::try_from(backing).map_err(|_| empty_capacity())?)
        .map_err(|stop| match stop {
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
            | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                empty_work()
            }
            _ => empty_capacity(),
        })?;
    let initialized = std::mem::size_of::<WorthQueryAdmittedOutputDemand<Schema, Family>>()
        .checked_add(std::mem::size_of::<RequiredFreshProgress<Schema>>())
        .and_then(|work| {
            work.checked_add(std::mem::size_of::<Box<dyn ErasedRequiredSuccessor<Schema>>>())
        })
        .and_then(|work| work.checked_add(std::mem::size_of::<Box<dyn Any>>()))
        .and_then(|work| {
            work.checked_add(std::mem::size_of::<
                Box<TypedRequiredSuccessor<Schema, Family>>,
            >())
        })
        .and_then(|work| work.checked_add(FAMILY_SUBJECT.len() + 4))
        .ok_or_else(empty_work)?;
    admission
        .charge_external_work(u64::try_from(initialized).map_err(|_| empty_work())?)
        .map_err(|_| empty_work())?;
    if progress.outcome.is_some()
        || progress.successor.concrete_type()
            != TypeId::of::<TypedRequiredSuccessor<Schema, Family>>()
    {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            FAMILY_SUBJECT,
        ));
    }
    Ok(())
}

fn empty_work() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_capacity() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "",
    )
}
