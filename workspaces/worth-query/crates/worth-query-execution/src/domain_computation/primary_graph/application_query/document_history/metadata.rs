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
pub(super) struct CompletedMetadataRead<'phase, 'scope> {
    source: pending::PendingDocumentRead,
    phase: &'phase WorthQueryAdvancementPhase<'scope>,
}
impl CompletedMetadataRead<'_, '_> {
    pub(super) fn confirm<S: ApplicationSchema, H: ApplicationHistoryBinding<S>, T>(
        self,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        value: T,
    ) -> Result<T, Denial> {
        if self.source.model() != document.model {
            return Err(Denial::ForeignModel);
        }
        if self.source.identity() != document.expected {
            return Err(Denial::HeadDiverged);
        }
        self.source
            .confirm(self.phase, document.application(), document.branch(), value)
            .map_err(|(_, cause)| cause)
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistoryRead<'_, S, H>
{
    /// Resolve an original typed metadata capability; it is freshly bound to this query session.
    pub fn with_metadata_capability<C, O, I>(
        mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
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
        mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        access: &WorthQueryProgramPublicationAccess,
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
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        let execution = phase
            .checked_execution_request_for(&runtime.product_runtime)
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
        let source = pending::PendingDocumentRead::capture(&self.plan);
        // The original binding owns projection, body/filter denials, completion
        // and cleanup. Its common current-security validation consumes this
        // carried decision before the original query body may read.
        let _read = runtime.read_document_history(phase, access, document, self)?;
        Ok(CompletedMetadataRead { source, phase })
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn read_document_history_step<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryStep, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed = read.complete_metadata_read(self, phase, document, access)?;
        completed.confirm(document, document.current_history_step(access).clone())
    }
}
