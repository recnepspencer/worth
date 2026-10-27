use super::{
    WorthQueryInstalledOwnedAsyncSource, WorthQueryOwnedAsyncRequestCustody as Custody,
    WorthQueryOwnedAsyncRuntimeCause as Cause, WorthQueryOwnedAsyncRuntimeDenial as Denial,
    WorthQueryOwnedAsyncRuntimeServices, WorthQueryOwnedAsyncRuntimeStop as Stop,
};
use worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial as WorldDenial;

impl WorthQueryOwnedAsyncRuntimeServices<'_> {
    pub fn admit(
        &self,
        source: &WorthQueryInstalledOwnedAsyncSource,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Custody, Stop> {
        self.admit_from_source(source, selected, || {
            self.world
                .admit_unqueued_owned_async_request(self.bridge, &source.lowered, selected)
        })
    }

    /// Retains an observed snapshot, not authoritative committed-source meaning.
    pub fn admit_at_branch_head(
        &self,
        source: &WorthQueryInstalledOwnedAsyncSource,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Custody, Stop> {
        self.admit_from_source(source, selected, || {
            self.world
                .admit_unqueued_owned_async_request_at_branch_head(
                    self.bridge,
                    &source.lowered,
                    selected,
                )
        })
    }

    fn admit_from_source(
        &self,
        source: &WorthQueryInstalledOwnedAsyncSource,
        selected: &crate::basis::WorthQueryProductBranchLease,
        admit: impl FnOnce() -> Result<
            worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
            WorldDenial,
        >,
    ) -> Result<Custody, Stop> {
        if source.runtime != self.runtime {
            return Err(Denial::ForeignRuntime.into());
        }
        if source.graph != self.bridge.owned_signal_graph_instance_id() {
            return Err(Denial::SuccessorRuntime.into());
        }
        self.require_installed(&source.lowered)?;
        self.require_selected(selected)?;
        let native = admit().map_err(|cause| match cause {
            WorldDenial::ForeignOwner | WorldDenial::RelationalSourceMismatch => {
                Denial::ProductSelectionMismatch
            }
            WorldDenial::NoCommittedSource => Denial::NoCommittedSource,
            WorldDenial::Bridge(cause) => Denial::Request(cause),
        })?;
        let owner = Custody {
            runtime: self.runtime,
            graph: source.graph,
            native,
        };
        if let Some(cause) = owner.native.refusal() {
            return Err(Stop::after_admission(Cause::Custody(cause), owner));
        }
        if let Err(cause) = owner.witness() {
            return Err(Stop::after_admission(cause, owner));
        }
        Ok(owner)
    }
}
