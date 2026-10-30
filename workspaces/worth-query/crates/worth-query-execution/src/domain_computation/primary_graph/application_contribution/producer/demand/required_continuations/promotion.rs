//! Extract only the actual typed successor after the caller's exact join.

use super::super::WorthQueryOutputDemandDenialKind;
use super::*;

const FAMILY_SUBJECT: &str = "required successor family differs from caller demand";

impl<Schema> RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// A denial returns the same move-only Progress and its live Interest.
    /// The caller completes its Ready and registry joins before invoking this.
    pub(super) fn into_typed_admitted<Family>(
        self,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, (Self, WorthQueryOutputDemandDenial)>
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
        WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    {
        if let Err(stop) = prepare_typed_extraction::<Schema, Family>(&self) {
            return Err((self, stop));
        }
        let Self {
            outcome,
            successor,
            contacts: _,
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
    ) -> Result<Option<WorthQueryAdmittedOutputDemand<Schema, Family>>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
        WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    {
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
        if !continues_caller && !progress.predecessor().same_ready_cell(caller_ready)? {
            return Ok(None);
        }
        if !registry.required_successor_is_live(caller_ready, progress.interest())? {
            return Ok(None);
        }
        if !selected_ready.matches_interest(progress.interest())? {
            return Ok(None);
        }
        progress.successor.validate_for_current_handoff()?;

        if !progress.successor.continuations_empty() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "",
            ));
        }
        let progress = self
            .entries
            .pop()
            .expect("checked last installed required successor");
        let typed = match progress.into_typed_admitted::<Family>() {
            Ok(typed) => typed,
            Err((progress, denial)) => {
                self.entries.push(progress);
                return Err(denial);
            }
        };
        debug_assert!(typed.required_continuations.entries.is_empty());
        debug_assert!(typed.required_continuations.requested.is_empty());
        Ok(Some(typed))
    }
}

fn prepare_typed_extraction<Schema, Family>(
    progress: &RequiredFreshProgress<Schema>,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema> + 'static,
{
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
