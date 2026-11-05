//! Current control custody is lent only within the freshly admitted advancement.
use super::super::authority_lifetime::{HistoryAuthorityLifetime, OwnedHistoryAuthorityLifetime};
use super::*;
use crate::basis::{WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease};
use crate::domain_computation::authorization::{
    admitted_request::AdmittedRequestCurrentness, WorthQueryOperationAuthorizationDenial,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_history::{
    ApplicationHistoryBinding, ApplicationHistoryModel, ApplicationHistoryNavigationBinding,
};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, WorthQueryInstalledDocumentHistory,
};
use {
    crate::domain_computation::primary_graph::application_query::CompletedMetadataRead,
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation,
    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

pub(crate) mod producer_invocation;
pub(in crate::domain_computation::primary_graph) use producer_invocation::{
    HistoryBorrowedReadStop, HistoryProducerCutover, HistoryProducerInvocation,
};

/// This is a dependency on current C, not authority from historical D. It is
/// neither constructible nor retainable by the SDK; each callback gets a fresh
/// instance after actual READ, metadata and NAV admission.
pub struct WorthQueryHistoryOutputAuthority<'phase, 'scope> {
    phase: &'phase WorthQueryAdvancementPhase<'scope>,
    runtime: u64,
    binding: ApplicationSchemaBindingIdentity,
    profile: WorthQueryHistoryOutputProfile,
    current: WorthQueryProductObservationLease,
    original: Arc<WorthQueryApplicationReadObservation>,
    request: AdmittedRequestCurrentness,
    principal: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    selection: HistoryAuthorityLifetime,
    callback: HistoryAuthorityLifetime,
}

impl WorthQueryHistoryOutputAuthority<'_, '_> {
    pub(in crate::domain_computation::primary_graph) fn producer_invocation(
        &self,
    ) -> HistoryProducerInvocation<'_> {
        HistoryProducerInvocation::new(
            &self.current,
            &self.request,
            &self.principal,
            &self.selection,
            &self.callback,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn validate<S: ApplicationSchema>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<S>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        if !std::ptr::eq(phase, self.phase)
            || self.runtime != runtime.authority_identity().as_u64()
            || self.binding != runtime.installed_schema().binding_identity()
            || !self.request.same_request(request)
        {
            return Err(Denial::ForeignDocument);
        }
        phase
            .check_live_for(runtime.product_runtime())
            .map_err(Denial::Stopped)?;
        if let Some(lapse) = self.request.lapse() {
            return Err(Denial::Authorization(
                WorthQueryOperationAuthorizationDenial::new(
                    lapse.denial_kind(),
                    "history output request",
                ),
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn current(
        &self,
    ) -> &WorthQueryProductObservationLease {
        &self.current
    }

    pub(in crate::domain_computation::primary_graph) fn original(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.original
    }

    pub(in crate::domain_computation::primary_graph) fn request(&self) -> &WorthQueryRequestScope {
        self.request.request()
    }

    pub(in crate::domain_computation::primary_graph) fn profile(
        &self,
    ) -> &WorthQueryHistoryOutputProfile {
        &self.profile
    }

    pub(in crate::domain_computation::primary_graph) fn matches_read_access<S, A, I, E>(
        &self,
        access: &crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext<
            '_,
            S,
            A,
            I,
            E,
        >,
    ) -> bool {
        self.profile.model == super::super::HistoryModelIdentity::from_resolved(access.scope())
            && self.matches_read_principal(access)
    }

    pub(in crate::domain_computation::primary_graph) fn matches_read_principal<S, A, I, E>(
        &self,
        access: &crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext<
            '_,
            S,
            A,
            I,
            E,
        >,
    ) -> bool {
        self.principal == crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                access.principal().principal_entity_id(),
            )
    }

    /// Only move an already prepared readonly result at this delivery point.
    /// The original Native/World current-head receiver owns the handoff. No
    /// SDK callback, producer work or operational cleanup runs under its guards.
    pub(in crate::domain_computation::primary_graph) fn confirm_read<S: ApplicationSchema, T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<S>,
        request: &WorthQueryRequestScope,
        value: T,
    ) -> Result<T, WorthQueryHistoryNavigationDenial> {
        self.validate(phase, runtime, request)?;
        self.producer_invocation()
            .confirm_read(phase, runtime, value)
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    super::super::WorthQueryDocumentHistory<'runtime, S, H>
{
    pub(in crate::domain_computation::primary_graph) fn with_output_authority<
        'scope,
        N: ApplicationHistoryNavigationBinding<S, H>,
        Output,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        read: &CompletedMetadataRead<'_, 'scope>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        admission: &WorthQueryAdmittedApplicationOperation<
            S,
            N::Operation,
            N::Input,
            ApplicationHistoryModel<S, H>,
        >,
        profile: &WorthQueryHistoryOutputProfile,
        original: &Arc<WorthQueryApplicationReadObservation>,
        callback: impl FnOnce(&WorthQueryHistoryOutputAuthority<'_, 'scope>) -> Output,
    ) -> Result<Output, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        self.validate_navigation::<N>(phase, read, installed, admission)?;
        if profile.model != self.model
            || !original
                .belongs_to_selected_occurrence(self.application(), original.product_observation())
        {
            return Err(Denial::ForeignDocument);
        }
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let selection = {
            let mut selected = self.selection.lock();
            if selected.expected
                != WorthQueryProductBranchReadIdentity::from_observation(
                    read.current_product().observation(),
                )
            {
                return Err(Denial::HeadChanged);
            }
            selected
                .retain_output_authority_lifetime(&execution)
                .map_err(Denial::Document)?
        };
        let callback_owner =
            OwnedHistoryAuthorityLifetime::reserve(&execution).map_err(|cause| {
                Denial::Document(super::super::WorthQueryDocumentHistoryDenial::Recording(
                    cause,
                ))
            })?;
        let authority = WorthQueryHistoryOutputAuthority {
            phase,
            runtime: self.application().runtime.authority_identity().as_u64(),
            binding: self.binding.clone(),
            profile: profile.clone(),
            current: read.current_product().retained_clone(),
            original: Arc::clone(original),
            request: admission.retain_admitted_request(),
            principal: read.principal(),
            selection,
            callback: callback_owner.retain(),
        };
        read.confirm(self, ()).map_err(Denial::Document)?;
        let output = callback(&authority);
        drop(callback_owner);
        Ok(output)
    }
}
