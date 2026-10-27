use super::super::{signal_port::BridgeConditionalSignalPort, BridgeOwnedSignalRuntime};
use super::{
    BridgeOwnedAsyncManagedDenial as Denial, BridgeOwnedAsyncManagedRequestAdmission as Custodian,
};

impl BridgeOwnedSignalRuntime {
    pub(super) fn retain_managed_native_progress(
        &self,
        owner: &mut Custodian,
        port: BridgeConditionalSignalPort,
        business: Option<crate::facade::BridgeAsyncRequestAdmissionRequest>,
    ) -> Result<(), Denial> {
        // Native has already retained its exact origin, occurrence and any debt.
        // Install the matching port before fallible descriptive Bridge assembly.
        owner.port = port;
        owner.request = None;
        if let Some(business) = business {
            owner.business = Some(business);
        }
        self.refresh_managed_owned_async_receipt(owner);
        match owner.refusal() {
            Some(cause) => Err(cause),
            None => Ok(()),
        }
    }

    pub fn finish_managed_owned_async_wake_cleanup(
        &self,
        owner: &mut Custodian,
    ) -> Result<(), Denial> {
        owner.require_owner(&self.async_observation_authority)?;
        owner
            .port
            .finish_managed_owned_async_wake_cleanup(&mut owner.native)
            .map_err(Denial::Service)?;
        self.refresh_managed_owned_async_receipt(owner);
        match owner.refusal() {
            Some(cause) => Err(cause),
            None => Ok(()),
        }
    }
}
