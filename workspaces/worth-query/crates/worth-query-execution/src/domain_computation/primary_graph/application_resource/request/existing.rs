//! Opaque progression and exact-selector Close never borrow Inspect permission.
use super::super::{
    action,
    selector::{WorthQueryApplicationResource, WorthQueryApplicationResourceReceipt},
    stop::WorthQueryResourceActionDenial,
    storage::ResourceStorage,
};
use super::*;
use worth_query_installation::facade::WorthQueryResourceAccessPurpose as Purpose;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub fn close<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
    ) -> Result<super::super::closed::WorthQueryClosedResource, WorthQueryResourceActionDenial>
    {
        self.close_with(handle, Purpose::Close, |_, _, _| Ok(None))
    }

    pub fn close_governed<Binding, Capability, Operation, Input>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<super::super::closed::WorthQueryClosedResource, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.close_with(handle, Purpose::Close, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    pub fn close_detailed<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
    ) -> Result<super::super::closed::WorthQueryClosedResource, WorthQueryResourceActionDenial>
    {
        self.close_with(handle, Purpose::CloseDetail, |_, _, _| Ok(None))
    }

    pub fn close_detailed_governed<Binding, Capability, Operation, Input>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<super::super::closed::WorthQueryClosedResource, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.close_with(
            handle,
            Purpose::CloseDetail,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
        )
    }

    fn close_with<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        purpose: Purpose,
        pending: impl FnOnce(
            &InstalledResourceBinding<Schema, Binding>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
    ) -> Result<super::super::closed::WorthQueryClosedResource, WorthQueryResourceActionDenial>
    {
        let selector = handle.receipt();
        let mut closed = None;
        let result = self.open(|execution| {
            let installed = self.installed::<Binding>()?;
            action::with_existing(
                self.runtime,
                execution,
                self.external,
                self.request,
                &installed,
                &selector,
                purpose,
                |principal, scope| pending(&installed, principal, scope),
                ResourceStorage::take,
                |row, _| {
                    super::super::progression::close::close(
                        self.runtime,
                        execution,
                        &installed,
                        row,
                        false,
                    )
                },
                |row, ()| match row.recycle() {
                    Ok(mut evidence) => {
                        if purpose == Purpose::CloseDetail {
                            // These facts follow actual provider/queue/Native
                            // cleanup and this exact admitted CloseDetail leaf.
                            evidence.detail = Some(super::super::WorthQueryResourceCleanupDetail {
                                provider_closed: true,
                                queue_drained: true,
                                native_released: true,
                            });
                        }
                        handle.closed();
                        closed = Some(evidence);
                        Ok(())
                    }
                    Err((row, cause)) => {
                        drop(row);
                        Err(ResourceCause::Storage(cause))
                    }
                },
            )
        });
        match result {
            Ok(()) => Ok(closed.expect("successful Close retains actual cleanup evidence")),
            Err(cause) => Err(WorthQueryResourceActionDenial { cause, closed }),
        }
    }

    pub fn reclaim<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        super::super::WorthQueryResourceStartStop<Schema, Binding>,
    > {
        self.reclaim_with(selector, |_, _, _| Ok(None))
    }

    pub fn reclaim_governed<Binding, Capability, Operation, Input>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        super::super::WorthQueryResourceStartStop<Schema, Binding>,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.reclaim_with(selector, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    fn reclaim_with<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        pending: impl FnOnce(
            &InstalledResourceBinding<Schema, Binding>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        super::super::WorthQueryResourceStartStop<Schema, Binding>,
    > {
        let mut handle = None;
        let result = self.open(|execution| {
            let installed = self.installed::<Binding>()?;
            action::with_existing(
                self.runtime,
                execution,
                self.external,
                self.request,
                &installed,
                selector,
                Purpose::Reclaim,
                |principal, scope| pending(&installed, principal, scope),
                ResourceStorage::take_abandoned,
                |_, _| Ok(()),
                |row, ()| {
                    installed
                        .storage
                        .claim(row.key())
                        .map_err(ResourceCause::Storage)?;
                    handle = Some(WorthQueryApplicationResource::issued(
                        &installed,
                        row.key(),
                        Arc::clone(&selector.access),
                    ));
                    drop(row);
                    Ok(())
                },
            )
        });
        match result {
            Ok(()) => Ok(handle.expect("successful exact reclaim owns its actual row")),
            Err(cause) => Err(super::super::WorthQueryResourceStartStop {
                handle,
                denial: cause.into(),
            }),
        }
    }
}
