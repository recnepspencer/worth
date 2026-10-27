//! Exact external recovery custody remains with its original9.17.7 owner.
use super::super::{
    action, storage::ResourceStorage, WorthQueryClosedResource, WorthQueryResourceActionDenial,
    WorthQueryResourceRecoveryCloseStop,
};
use super::super::{
    selector::WorthQueryApplicationResource, WorthQueryResourceOwnedEffectStartStop,
};
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryInspectAuthority,
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryScopeResolution,
    application_resource::ApplicationResourceInput,
};
use worth_query_installation::facade::WorthQueryResourceAccessPurpose as Purpose;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub fn advance_for_owned_effect<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        authority: &WorthQueryRecoveryInspectAuthority,
    ) -> Result<(), WorthQueryResourceActionDenial> {
        self.advance_effect(handle, authority, |_, _, _| Ok(None))
    }

    pub fn advance_for_owned_effect_governed<Binding, Capability, Operation, Input>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        authority: &WorthQueryRecoveryInspectAuthority,
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
        self.advance_effect(handle, authority, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    fn advance_effect<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        authority: &WorthQueryRecoveryInspectAuthority,
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
                    if row.occurrence().recovery.is_none() {
                        return Err(ResourceCause::RecoveryHandoffRequired);
                    }
                    super::super::progression::advance(
                        self.runtime,
                        execution,
                        &installed,
                        row,
                        selected,
                        Some(authority),
                    )
                },
                |row, ()| {
                    drop(row);
                    Ok(())
                },
            )
        })
        .map_err(Self::denial)
    }

    pub fn close_for_effect_recovery<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        authority: &WorthQueryRecoveryInspectAuthority,
    ) -> Result<
        (WorthQueryRecoveryHandle, WorthQueryClosedResource),
        WorthQueryResourceRecoveryCloseStop,
    > {
        self.close_effect(handle, authority, |_, _, _| Ok(None))
    }

    pub fn close_for_effect_recovery_governed<Binding, Capability, Operation, Input>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        authority: &WorthQueryRecoveryInspectAuthority,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<
        (WorthQueryRecoveryHandle, WorthQueryClosedResource),
        WorthQueryResourceRecoveryCloseStop,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.close_effect(handle, authority, |installed, principal, scope| {
            self.pending(installed, principal, scope, capability)
        })
    }

    fn close_effect<Binding: ApplicationResourceBinding<Schema>>(
        &self,
        handle: &mut WorthQueryApplicationResource<Schema, Binding>,
        authority: &WorthQueryRecoveryInspectAuthority,
        pending: impl FnOnce(
            &InstalledResourceBinding<Schema, Binding>,
            EntityId,
            EntityId,
        ) -> Result<
            Option<WorthQueryPendingApplicationQueryGovernance>,
            ResourceCause,
        >,
    ) -> Result<
        (WorthQueryRecoveryHandle, WorthQueryClosedResource),
        WorthQueryResourceRecoveryCloseStop,
    > {
        let selector = handle.receipt();
        let mut returned = None;
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
                Purpose::Close,
                |principal, scope| pending(&installed, principal, scope),
                ResourceStorage::take,
                |row, _| {
                    let occurrence = row.occurrence();
                    let recovery = occurrence
                        .recovery
                        .as_ref()
                        .ok_or(ResourceCause::RecoveryHandoffRequired)?;
                    super::super::recovery::require_authority(
                        self.runtime,
                        recovery,
                        authority,
                        &occurrence.access,
                        &occurrence.business_product,
                    )?;
                    super::super::progression::close::close(
                        self.runtime,
                        execution,
                        &installed,
                        row,
                        true,
                    )
                },
                |mut row, ()| {
                    let recovery = row
                        .occurrence_mut()
                        .recovery
                        .take()
                        .expect("actual cleanup retained the original handle");
                    match row.recycle() {
                        Ok(evidence) => {
                            handle.closed();
                            returned = Some(recovery);
                            closed = Some(evidence);
                            Ok(())
                        }
                        Err((mut row, cause)) => {
                            row.occurrence_mut().recovery = Some(recovery);
                            drop(row);
                            Err(ResourceCause::Storage(cause))
                        }
                    }
                },
            )
        });
        match result {
            Ok(()) => Ok((
                returned.expect("successful authorized cleanup returns exact recovery custody"),
                closed.expect("successful cleanup retains its actual evidence"),
            )),
            Err(cause) => Err(WorthQueryResourceRecoveryCloseStop {
                recovery: returned,
                denial: WorthQueryResourceActionDenial { cause, closed },
            }),
        }
    }

    pub fn start_for_owned_effect<Binding>(
        &self,
        input: ApplicationResourceInput<Schema, Binding>,
        recovery: WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryInspectAuthority,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        WorthQueryResourceOwnedEffectStartStop<Schema, Binding>,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        <Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let mut recovery = Some(recovery);
        self.start_with::<Binding>(&input, &mut recovery, Some(authority), |_, _, _| Ok(None))
            .map_err(|resource| WorthQueryResourceOwnedEffectStartStop { resource, recovery })
    }

    pub fn start_for_owned_effect_governed<Binding, Capability, Operation, Input>(
        &self,
        input: ApplicationResourceInput<Schema, Binding>,
        recovery: WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryInspectAuthority,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<
        WorthQueryApplicationResource<Schema, Binding>,
        WorthQueryResourceOwnedEffectStartStop<Schema, Binding>,
    >
    where
        Binding: ApplicationResourceBinding<Schema>,
        <Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        let mut recovery = Some(recovery);
        self.start_with::<Binding>(
            &input,
            &mut recovery,
            Some(authority),
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
        )
        .map_err(|resource| WorthQueryResourceOwnedEffectStartStop { resource, recovery })
    }
}
