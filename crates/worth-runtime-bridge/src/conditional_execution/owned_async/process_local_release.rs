use super::super::BridgeOwnedSignalRuntime;
use super::{
    BridgeOwnedAsyncCommittedCompletion, BridgeOwnedAsyncManagedDenial as Denial,
    BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};
use std::sync::Arc;

/// Exact Native release evidence, without a live request cleanup capability.
pub struct BridgeOwnedAsyncReleasedResource {
    native: worth_signal::facade::branch::SignalOwnedAsyncReleasedResource,
}

pub struct BridgeOwnedAsyncResourceReleaseRefusal {
    owner: Custodian,
    completion: Option<BridgeOwnedAsyncCommittedCompletion>,
    cause: Denial,
}

impl BridgeOwnedAsyncReleasedResource {
    pub fn native(&self) -> &worth_signal::facade::branch::SignalOwnedAsyncReleasedResource {
        &self.native
    }
}

impl BridgeOwnedAsyncResourceReleaseRefusal {
    pub fn cause(&self) -> &Denial {
        &self.cause
    }

    pub fn into_parts(
        self,
    ) -> (
        Custodian,
        Option<BridgeOwnedAsyncCommittedCompletion>,
        Denial,
    ) {
        (self.owner, self.completion, self.cause)
    }
}

impl BridgeOwnedSignalRuntime {
    pub fn abort_process_local_owned_async_retry(
        &self,
        owner: &mut Custodian,
    ) -> Result<(), Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        owner
            .port
            .abort_process_local_owned_async_retry(&mut owner.native)
            .map_err(Denial::Signal)
    }

    pub fn release_process_local_managed_request(
        &self,
        mut owner: Custodian,
        completion: Option<BridgeOwnedAsyncCommittedCompletion>,
    ) -> Result<BridgeOwnedAsyncReleasedResource, BridgeOwnedAsyncResourceReleaseRefusal> {
        let affinity = owner
            .require_owner(&self.async_observation_authority)
            .and_then(|()| {
                if completion.as_ref().is_some_and(|value| {
                    !Arc::ptr_eq(&value.authority, &self.async_observation_authority)
                }) {
                    Err(Denial::ForeignOwner)
                } else {
                    Ok(())
                }
            });
        if let Err(cause) = affinity {
            return Err(BridgeOwnedAsyncResourceReleaseRefusal {
                owner,
                completion,
                cause,
            });
        }
        let completion_context = completion
            .as_ref()
            .map(|value| (Arc::clone(&value.authority), value.port.clone()));
        let native_completion = completion.map(|value| value.native);
        match owner
            .port
            .release_process_local_managed_request(owner.native, native_completion)
        {
            Ok(native) => Ok(BridgeOwnedAsyncReleasedResource { native }),
            Err(refusal) => {
                let (native, completion, cause) = refusal.into_parts();
                owner.native = native;
                let completion = completion.map(|native| {
                    let (authority, port) =
                        completion_context.expect("actual completion keeps its owner");
                    BridgeOwnedAsyncCommittedCompletion {
                        authority,
                        port,
                        native,
                    }
                });
                Err(BridgeOwnedAsyncResourceReleaseRefusal {
                    owner,
                    completion,
                    cause: Denial::Signal(cause),
                })
            }
        }
    }

    pub fn release_process_local_completion_denial(
        &self,
        owner: &Custodian,
        completion: &super::super::BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<(), Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        if !completion.is_owned_by(&self.async_observation_authority) {
            return Err(Denial::ForeignOwner);
        }
        // The opaque Native admission retains the originating service itself.
        let native = completion.native_admission();
        owner
            .port
            .release_process_local_completion_denial(native)
            .map_err(Denial::Service)
    }
}
