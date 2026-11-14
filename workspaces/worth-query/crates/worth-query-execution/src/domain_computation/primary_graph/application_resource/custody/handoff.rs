//! Home's exclusive handoff releases each original installed binding.
use super::super::super::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
};
use super::super::{
    progression::close, storage::ResourceStorageDenial, WorthQueryResourceFrameKind,
};
use super::*;

pub(in crate::domain_computation::primary_graph::application_resource) trait HomeResourceCustody<S>:
    Send + Sync
{
    fn handoff(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial>;
}
impl<S: ApplicationSchema, B: ApplicationResourceBinding<S>> HomeResourceCustody<S>
    for InstalledResourceBinding<S, B>
{
    fn handoff(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        for key in self.storage.allocated_keys() {
            let mut row = self
                .storage
                .take(key)
                .map_err(|cause| refusal(ResourceCause::Storage(cause)))?;
            if row.occurrence().recovery.is_some() {
                return Err(refusal(ResourceCause::RecoveryHandoffRequired));
            }
            if row.occurrence().disposal != crate::domain_computation::primary_graph::query_obligations::history::ResourceDisposal::Retained {
                super::discharge(runtime, request, &mut row).map_err(refusal)?;
                row.retire_to_home().map_err(|(_, cause)| refusal(ResourceCause::Storage(cause)))?;
                continue;
            }
            // Retention can have refused before a Native slot was allocated.
            // A retained row with no acknowledged record cannot hide at close.
            retain(runtime, request, self, &mut row).map_err(refusal)?;
            freeze(runtime, request, self, &mut row).map_err(refusal)?;
            close::close(runtime, request, self, &mut row, false).map_err(refusal)?;
            record_release(runtime, request, &mut row).map_err(refusal)?;
            row.retire_to_home()
                .map_err(|(_, cause)| refusal(ResourceCause::Storage(cause)))?;
        }
        Ok(())
    }
}
fn refusal(cause: ResourceCause) -> Denial {
    match cause {
        ResourceCause::Obligation(cause) => Denial::ManagedCleanup(cause),
        ResourceCause::Storage(ResourceStorageDenial::Busy) => {
            Denial::Busy(Owner::InstalledResources)
        }
        cause => Denial::Resource(super::super::stop::WorthQueryResourceActionDenial::from(
            cause,
        )),
    }
}
pub(in crate::domain_computation::primary_graph::application_resource) fn freeze<
    S: ApplicationSchema,
    B: ApplicationResourceBinding<S>,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<S, B>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause> {
    let Some(identity) = row.occurrence().obligation else {
        return Ok(());
    };
    if !row.occurrence().ingress_frozen {
        let occurrence = row.occurrence();
        let ingress = match (occurrence.native_slot, occurrence.attempt.as_ref()) {
            (Some(index), Some(attempt)) => installed.storage.nodes[index]
                .ingress
                .freeze_for_handoff(attempt)
                .map_err(ResourceCause::Ingress)?,
            _ => Vec::new(),
        };
        let result = occurrence.result.as_ref().map(|result| {
            (
                match result.kind {
                    WorthQueryResourceFrameKind::Result => QueryResourceResultKind::Result,
                    WorthQueryResourceFrameKind::DomainFailure => {
                        QueryResourceResultKind::DomainFailure
                    }
                    WorthQueryResourceFrameKind::EffectsIndeterminate => {
                        QueryResourceResultKind::EffectsIndeterminate
                    }
                },
                result.value.value.clone(),
            )
        });
        let occurrence = row.occurrence_mut();
        let handoff = occurrence
            .handoff
            .get_or_insert_with(QueryResourceHandoff::default);
        handoff.ingress = ingress;
        handoff.result = result;
        if let ResourceLifecycle::Live { owner, .. } = &occurrence.lifecycle {
            handoff
                .original_request
                .get_or_insert(owner.request_handle());
            handoff.request = Some(owner.request_handle());
        }
        occurrence.ingress_frozen = true;
    }
    runtime
        .installed_program_support()
        .ok_or(ResourceCause::Unavailable)?
        .lifecycle()
        .record_resource_handoff(
            request,
            identity,
            row.occurrence()
                .handoff
                .clone()
                .expect("frozen original ingress"),
        )
        .map_err(ResourceCause::Obligation)
}
pub(in crate::domain_computation::primary_graph::application_resource) fn record_release<
    S: ApplicationSchema,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &ExecutionRequest<'_, '_>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause> {
    let Some(identity) = row.occurrence().obligation else {
        return Ok(());
    };
    let occurrence = row.occurrence_mut();
    if !occurrence.provider_closed || occurrence.candidate.is_some() {
        return Err(ResourceCause::Storage(
            ResourceStorageDenial::CleanupPending,
        ));
    }
    let release = match &occurrence.lifecycle {
        ResourceLifecycle::Released(released) => QueryResourceRelease {
            terminal: Some(released.native().terminal().clone()),
            predecessor: released.native().released_predecessor().cloned(),
        },
        ResourceLifecycle::Reserved { .. } if occurrence.attempt.is_none() => {
            QueryResourceRelease {
                terminal: None,
                predecessor: None,
            }
        }
        _ => {
            return Err(ResourceCause::Storage(
                ResourceStorageDenial::CleanupPending,
            ))
        }
    };
    let handoff = occurrence
        .handoff
        .get_or_insert_with(QueryResourceHandoff::default);
    if let Some(terminal) = &release.terminal {
        handoff.request = Some(terminal.handle());
    }
    handoff.release = Some(release);
    runtime
        .installed_program_support()
        .ok_or(ResourceCause::Unavailable)?
        .lifecycle()
        .record_resource_handoff(request, identity, handoff.clone())
        .map_err(ResourceCause::Obligation)
}
