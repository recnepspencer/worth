//! Follow the retained native refresh of an exact predecessor before recertifying it.
use super::*;

pub(in super::super) enum OwnedRefreshStop {
    Selection(WorthQueryOutputDemandDenial),
    Recorded(WorthQueryOutputDemandDenial),
}
impl<Schema: ApplicationSchema + 'static> RequiredContinuations<Schema> {
    pub(in super::super) fn refresh_position(
        &self,
        selected: &SelectedReadyReadmission,
    ) -> Result<Option<usize>, WorthQueryOutputDemandDenial> {
        for (index, entry) in self.entries.iter().enumerate().rev() {
            if entry.predecessor().same_ready_cell(selected)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
}

/// Nothing here makes a predecessor current or reopens a stopped row. The
/// original typed successor resumes under its actual request and commit mode;
/// native supersession alone can give its unpublished occurrence back.
pub(in super::super) fn resume_owned_refresh<Schema: ApplicationSchema + 'static>(
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    invocation: &ProducerInvocationAuthority<'_>,
    custody: held::ContinuationCustody<'_, Schema>,
    selected: &SelectedReadyReadmission,

    performed: &std::cell::RefCell<PerformedMembers>,
) -> Result<Option<held::HeldUpstream>, OwnedRefreshStop> {
    let (custody, contacts) = match custody {
        held::ContinuationCustody::Caller(custody, contacts) => (custody, Some(contacts)),
        held::ContinuationCustody::Queue(custody) => (custody, None),
    };
    if custody
        .refresh_position(selected)
        .map_err(OwnedRefreshStop::Selection)?
        .is_none()
    {
        return Ok(None);
    }
    // A reopened attempt may have returned exactly the Ready it replaced.
    // Existing owner cleanup ends that entry before another claim of the Ready.
    custody
        .end_restored(runtime.output_production().demands())
        .map_err(OwnedRefreshStop::Selection)?;
    let Some(index) = custody
        .refresh_position(selected)
        .map_err(OwnedRefreshStop::Selection)?
    else {
        return Ok(Some(held::HeldUpstream::GaveBack));
    };
    held::resume_local(
        phase, runtime, principal, request, branch, custody, contacts, index, invocation, performed,
    )
    .map(Some)
    .map_err(|stop| {
        // Native execution failures belong to the actual successor, never to
        // the predecessor Ready retained only for scheduling and handoff.
        if !matches!(
            stop.kind(),
            crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ExecutionRequest(_)
        ) {
            runtime
                .output_production()
                .demands()
                .record_required_stop(custody.entries[index].interest().key(), &stop);
        }
        OwnedRefreshStop::Recorded(stop)
    })
}
