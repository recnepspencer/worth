//! Advancement returns opaque custody; it does not implicitly disclose state.
use super::super::{
    action, selector::WorthQueryApplicationResource, stop::WorthQueryResourceActionDenial,
    storage::ResourceStorage,
};
use super::*;
use worth_query_installation::facade::WorthQueryResourceAccessPurpose as Purpose;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub fn advance<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
    ) -> Result<(), WorthQueryResourceActionDenial> {
        self.progress(handle, false, |_, _, _| Ok(None))
    }

    pub fn advance_governed<Binding, Capability, Operation, Input>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<(), WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.progress(handle, false, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    /// Uses the existing Native freshness/revalidation owner. Original input,
    /// parameters and original business Product remain the same association.
    pub fn revalidate<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
    ) -> Result<(), WorthQueryResourceActionDenial> {
        self.progress(handle, true, |_, _, _| Ok(None))
    }

    pub fn revalidate_governed<Binding, Capability, Operation, Input>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<(), WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.progress(handle, true, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    fn progress<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        revalidate: bool,
        pending: impl FnOnce(
            &InstalledResourceBinding<Schema, Binding>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
    ) -> Result<(), WorthQueryResourceActionDenial> {
        self.open(|execution| {
            let installed = self.installed::<Binding>()?;
            action::with_existing(
                self.runtime,
                execution,
                self.external,
                self.request,
                &installed,
                handle.selector(),
                Purpose::Advance,
                |principal, scope| pending(&installed, principal, scope),
                ResourceStorage::take,
                |row, selected| {
                    if revalidate {
                        super::super::progression::revalidate(
                            self.runtime,
                            execution,
                            &installed,
                            row,
                            selected,
                        )
                    } else {
                        super::super::progression::advance(
                            self.runtime,
                            execution,
                            &installed,
                            row,
                            selected,
                            None,
                        )
                    }
                },
                |row, ()| {
                    drop(row);
                    Ok(())
                },
            )
        })
        .map_err(Self::denial)
    }
}
