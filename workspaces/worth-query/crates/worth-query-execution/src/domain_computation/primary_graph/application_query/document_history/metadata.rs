//! Current history metadata is disclosed by its own decision on the real READ session.
use super::super::{
    admission::prepare_governed_access, history::admit_metadata,
    WorthQueryApplicationHistoryDenial as MetadataDenial, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};
use super::*;
use crate::domain_computation::authorization::application_disclosure::contract::compile_document_history_disclosure_contract;
use crate::domain_computation::primary_graph::WorthQueryHistoryStep;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

/// Only a completed ordinary READ with the independent metadata decision can
/// create this final correspondence check. It stays within its original phase.
pub(in crate::domain_computation::primary_graph) struct CompletedMetadataRead<'phase, 'scope> {
    source: pending::PendingDocumentRead,
    phase: &'phase WorthQueryAdvancementPhase<'scope>,
    history: std::any::TypeId,
    principal: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
}
impl CompletedMetadataRead<'_, '_> {
    pub(in crate::domain_computation::primary_graph) fn current_product(
        &self,
    ) -> &crate::basis::WorthQueryProductObservationLease {
        self.source.product()
    }
    pub(in crate::domain_computation::primary_graph) fn phase(
        &self,
    ) -> &WorthQueryAdvancementPhase<'_> {
        self.phase
    }
    pub(in crate::domain_computation::primary_graph) fn principal(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        self.principal
    }
    pub(super) fn source(&self) -> &pending::PendingDocumentRead {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn confirm<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        T,
    >(
        &self,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        value: T,
    ) -> Result<T, Denial> {
        if self.history != crate::equality_type_id::EqualityTypeId::of::<H>().into_external() {
            return Err(Denial::InstalledBindingMismatch);
        }
        if self.source.model() != document.model {
            return Err(Denial::ForeignModel);
        }
        self.source.confirm_selection(self.phase, document, value)
    }

    pub(super) fn confirm_generation<S: ApplicationSchema, H: ApplicationHistoryBinding<S>, T>(
        &self,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        generation: u64,
        value: T,
    ) -> Result<T, Denial> {
        let value = self.confirm(document, value)?;
        let selected = document.selection.lock();
        if self.source.identity() != selected.expected {
            return Err(Denial::HeadDiverged);
        }
        if selected.journal.view_generation() != generation {
            return Err(Denial::Paging(
                crate::domain_computation::primary_graph::WorthQueryHistoryPageDenial::StaleCursor,
            ));
        }
        drop(selected);
        Ok(value)
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistoryRead<'_, S, H>
{
    /// Resolve an original typed metadata capability; it is freshly bound to this query session.
    pub fn with_metadata_capability<C, O, I>(
        mut self,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<Self, Denial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        let installed = runtime
            .installed_schema()
            .capability(capability, operation)
            .map_err(Denial::MetadataCapabilityInstallation)?;
        let selected = runtime
            .application_runtime()
            .on_branch(
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    self.plan.basis.selected_product().observation(),
                )
                .product_branch(),
            )
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        let capability = selected
            .admit_capability_access(
                phase,
                self.plan.principal,
                &installed,
                input,
                self.plan.controls.request_scope(),
            )
            .map_err(|cause| Denial::Metadata(MetadataDenial::Authorization(cause)))?;
        let access =
            WorthQueryApplicationQueryAccessContext::new(self.plan.principal, self.plan.scope);
        self.metadata = Some(
            prepare_governed_access(
                runtime,
                self.plan.query,
                &access,
                capability,
                self.plan.controls.request_scope(),
            )
            .map_err(|cause| Denial::Metadata(MetadataDenial::Query(cause)))?,
        );
        Ok(self)
    }

    pub(super) fn complete_metadata_read<'phase, 'scope>(
        self,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        access: &WorthQueryProgramPublicationAccess,
    ) -> Result<CompletedMetadataRead<'phase, 'scope>, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed = self.observe_metadata_read(runtime, phase, document, access)?;
        if completed.source.identity() != document.selection.lock().expected {
            return Err(Denial::HeadDiverged);
        }
        Ok(completed)
    }

    pub(super) fn observe_metadata_read<'phase, 'scope>(
        mut self,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        _access: &WorthQueryProgramPublicationAccess,
    ) -> Result<CompletedMetadataRead<'phase, 'scope>, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if !std::ptr::eq(runtime, document.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if runtime.installed_schema().binding_identity() != *self.history.binding_identity()
            || document.binding != *self.history.binding_identity()
            || self.binding.query().identity() != self.history.access_binding().query().identity()
            || self.plan.query.binding_observation().is_none()
            || self.plan.query.binding_observation()
                != self.history.access_binding().query().binding_observation()
            || self.binding.query().binding_observation() != self.plan.query.binding_observation()
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        let execution = phase
            .checked_execution_request_for(runtime.product_runtime())
            .map_err(Denial::Stopped)?;
        let contract =
            compile_document_history_disclosure_contract(self.history).map_err(|cause| {
                Denial::Metadata(MetadataDenial::Query(
                    WorthQueryApplicationQueryAdmissionDenial::new(
                        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                        cause.subject(),
                    ),
                ))
            })?;
        let metadata = admit_metadata::<H, _, _, _, _, _, _, _>(
            runtime,
            &execution,
            &mut self.plan,
            contract,
            self.metadata.take(),
            H::IDENTITY,
        )
        .map_err(Denial::Metadata)?;
        if !metadata.belongs_to::<H>() {
            return Err(Denial::InstalledBindingMismatch);
        }
        self.plan.history_metadata = Some(metadata);
        // The original binding owns projection, body/filter denials, completion
        // and cleanup. Its common current-security validation consumes this
        // carried decision before the original query body may read.
        let principal = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(self.plan.principal.principal_entity_id());
        let (source, _read) = self.execute(runtime, phase)?;
        if source.model() != document.model {
            return Err(Denial::ForeignModel);
        }
        source
            .confirm(phase, runtime, document.branch(), ())
            .map_err(|(_, cause)| cause)?;
        Ok(CompletedMetadataRead {
            source,
            phase,
            history: crate::equality_type_id::EqualityTypeId::of::<H>().into_external(),
            principal,
        })
    }
}

pub trait MetadataOps<S: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>
{
    fn read_document_history_step<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryStep, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed =
            read.complete_metadata_read(self.application_query_host(), phase, document, access)?;
        completed.confirm(document, document.current_history_step(access))
    }

    fn read_document_history_selection<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<crate::domain_computation::primary_graph::WorthQueryHistorySelection, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed =
            read.complete_metadata_read(self.application_query_host(), phase, document, access)?;
        let selection = document.current_history_selection(access);
        completed.confirm(document, selection)
    }
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                S,
            > + ?Sized,
    > MetadataOps<S> for HostRuntime
{
}
