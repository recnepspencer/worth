use super::{
    DeclarativeLiveQueryRequest, QuerySchemaView, WorthQueryAspectApiFinalizationCloseout,
    WorthQueryAuthoritativeMutationEvidenceCloseout,
    WorthQueryAuthoritativeMutationEvidenceSupport, WorthQueryBranchOptions,
    WorthQueryBranchSession, WorthQueryComputedBuilder, WorthQueryDerivedViewHandle,
    WorthQueryDerivedViewMaintainer, WorthQueryEffectBuilder, WorthQueryEffectHandle,
    WorthQueryEffectIntentReceipt, WorthQueryHandleContract, WorthQueryIntentDeclaration,
    WorthQueryIntentReceipt, WorthQueryLiveView, WorthQueryLiveViewBuilder,
    WorthQueryMutationSurfaceReport, WorthQueryPreviewOptions, WorthQueryPreviewSession,
    WorthQueryRuntime, WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimePublicApiContract, WorthQueryRuntimePublicApiFamilyContract,
    WorthQueryRuntimePublicSupportMatrix, WorthQueryRuntimeStateSnapshot,
    WorthQueryRuntimeStateTarget, WorthQueryWorkspaceLiveViewDeclaration,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::program::WorthQueryDerivedView;
use crate::session_label::WorthQuerySessionLabel;

mod conditional_execution;
mod graph_read_access;
mod invalidation_attachment;
mod invalidation_maintenance;
mod owned_async_source;
mod primary_source_rebind;
pub use primary_source_rebind::WorthQueryPrimaryGraphSourceRebindReceipt;
pub struct WorthQueryWorkspace {
    pub(super) name: String,
    pub(super) runtime: WorthQueryRuntime,
}

impl WorthQueryWorkspace {
    pub(crate) fn new(
        name: impl Into<String>,
        runtime: WorthQueryRuntime,
    ) -> Result<Self, WorthQueryRuntimeError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorthQueryRuntimeError::Workspace(
                crate::memory_workspace::WorthQueryWorkspaceError::new(
                    "workspace name may not be empty",
                ),
            ));
        }
        Ok(Self { name, runtime })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        self.runtime.current_snapshot_identity()
    }

    pub(crate) fn collection_entity(
        &self,
        collection: &str,
        identity: &crate::memory_workspace::WorthQueryEntityIdentity,
    ) -> super::WorthQueryBackendEntityLookup {
        self.runtime.backend.collection_entity(collection, identity)
    }

    pub(crate) fn runtime_authority_identity(&self) -> super::WorthQueryRuntimeAuthorityIdentity {
        self.runtime.authority_identity
    }

    pub(crate) fn capture_branch_comparison_basis(
        &self,
        label: WorthQuerySessionLabel,
    ) -> Result<super::WorthQueryRuntimeBranchComparisonBasis, WorthQueryRuntimeError> {
        self.runtime.capture_branch_comparison_basis(label)
    }

    pub(crate) fn capture_ordinary_mutation_authority(
        &self,
    ) -> Result<super::WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.runtime.capture_ordinary_mutation_authority()
    }

    pub(crate) fn capture_ordinary_preview_authority(
        &self,
        label: WorthQuerySessionLabel,
        effect_policy: super::WorthQueryEffectPolicy,
    ) -> Result<super::WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.runtime
            .capture_ordinary_preview_authority(label, effect_policy)
    }

    pub(crate) fn capture_ordinary_writeback_authority(
        &self,
    ) -> Result<super::WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.runtime.capture_ordinary_writeback_authority()
    }

    pub(crate) fn capture_ordinary_merge_authority(
        &self,
        target_branch: &super::WorthQueryAdmittedBranchName,
        source_branch: &super::WorthQueryAdmittedBranchName,
    ) -> Result<super::WorthQueryOrdinaryAuthorityAdmission, WorthQueryRuntimeError> {
        self.runtime
            .capture_ordinary_merge_authority(target_branch, source_branch)
    }

    pub(crate) fn ordinary_authority_drift(
        &self,
        admission: &super::WorthQueryOrdinaryAuthorityAdmission,
    ) -> super::WorthQueryOrdinaryAuthorityDrift {
        self.runtime.ordinary_authority_drift(admission)
    }

    pub(crate) fn validate_ordinary_merge_authority(
        &self,
        admission: super::WorthQueryOrdinaryAuthorityAdmission,
    ) -> Result<
        super::WorthQueryValidatedMergeAuthority,
        super::WorthQueryMergeAuthorityValidationError,
    > {
        self.runtime.validate_ordinary_merge_authority(admission)
    }

    pub(crate) fn admit_ordinary_rich_inspection(&self) -> Result<(), WorthQueryRuntimeError> {
        self.runtime.admit_ordinary_rich_inspection()
    }

    pub(crate) fn materialize_ordinary_inspection(
        &self,
        plan: &super::CausalInspectionPlan,
    ) -> Result<super::QueryCausalInspectionArtifact, super::WorthQueryBackendInspectionError> {
        self.runtime.materialize_ordinary_inspection(plan)
    }

    pub(crate) fn execute_ordinary_authoritative_mutation(
        &mut self,
        command: super::WorthQueryWriteCommand,
        materialize_inspection: bool,
    ) -> Result<super::WorthQueryLowerRuntimeMutationExecution, WorthQueryRuntimeError> {
        self.runtime
            .execute_ordinary_authoritative_mutation(command, materialize_inspection)
    }

    pub(crate) fn execute_ordinary_read_only_preview(
        &mut self,
        basis_admission: super::WorthQueryPreviewBasisAdmission,
        declaration_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        materialize_inspection: bool,
    ) -> Result<super::WorthQueryLowerRuntimePreviewExecution, WorthQueryRuntimeError> {
        self.runtime.execute_ordinary_read_only_preview(
            basis_admission,
            declaration_identity,
            materialize_inspection,
        )
    }

    pub(crate) fn execute_ordinary_preview_promotion(
        &mut self,
        basis_admission: super::WorthQueryPreviewBasisAdmission,
        declaration_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        command: super::WorthQueryWriteCommand,
        materialize_inspection: bool,
    ) -> Result<super::WorthQueryLowerRuntimePreviewExecution, WorthQueryRuntimeError> {
        self.runtime.execute_ordinary_preview_promotion(
            basis_admission,
            declaration_identity,
            command,
            materialize_inspection,
        )
    }

    pub(crate) fn execute_ordinary_writeback(
        &mut self,
        authority: super::WorthQueryOrdinaryAuthorityAdmission,
        declaration_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        materialize_inspection: bool,
    ) -> Result<
        super::WorthQueryLowerRuntimeWritebackExecution,
        super::WorthQueryOrdinaryWritebackExecutionError,
    > {
        self.runtime.execute_ordinary_writeback(
            authority,
            declaration_identity,
            materialize_inspection,
        )
    }

    pub(crate) fn execute_ordinary_merge(
        &mut self,
        authority: super::WorthQueryValidatedMergeAuthority,
        declaration_identity: &crate::evidence_identity::WorthQueryEvidenceIdentity,
        materialize_inspection: bool,
    ) -> Result<
        super::WorthQueryLowerRuntimeMergeExecution,
        super::WorthQueryOrdinaryMergeExecutionError,
    > {
        self.runtime
            .execute_ordinary_merge(authority, declaration_identity, materialize_inspection)
    }

    pub fn into_runtime(self) -> WorthQueryRuntime {
        self.runtime
    }

    pub fn public_api_contract(&self) -> WorthQueryRuntimePublicApiContract {
        self.runtime.public_api_contract()
    }

    pub fn public_handle_contract(&self) -> WorthQueryHandleContract {
        self.runtime.public_handle_contract()
    }

    pub fn public_support_matrix(&self) -> WorthQueryRuntimePublicSupportMatrix {
        self.runtime.public_support_matrix()
    }

    pub fn public_mutation_surface_report(&self) -> WorthQueryMutationSurfaceReport {
        self.runtime.public_mutation_surface_report()
    }

    pub fn public_authoritative_mutation_evidence_support(
        &self,
    ) -> WorthQueryAuthoritativeMutationEvidenceSupport {
        self.runtime
            .public_authoritative_mutation_evidence_support()
    }

    pub fn public_aspect_api_finalization_closeout(
        &self,
    ) -> WorthQueryAspectApiFinalizationCloseout {
        self.runtime.public_aspect_api_finalization_closeout()
    }

    pub fn public_authoritative_mutation_evidence_closeout(
        &self,
    ) -> WorthQueryAuthoritativeMutationEvidenceCloseout {
        self.runtime
            .public_authoritative_mutation_evidence_closeout()
    }

    pub fn admit_public_api_family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<WorthQueryRuntimePublicApiFamilyContract, WorthQueryRuntimeError> {
        self.runtime.admit_public_api_family(family)
    }

    pub fn state<T>(
        &self,
        target: T,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError>
    where
        T: WorthQueryRuntimeStateTarget,
    {
        target.into_state_snapshot(&self.runtime)
    }

    pub fn live_view<T>(
        &mut self,
        name: impl Into<String>,
        declaration: impl FnOnce(WorthQueryLiveViewBuilder) -> WorthQueryLiveViewBuilder,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        let name = name.into();
        let (request, schema_view) = declaration(WorthQueryLiveViewBuilder::surface(&name))
            .build()?
            .into_parts();
        self.runtime.declare_live_view(name, request, schema_view)
    }

    pub fn live_view_request<T>(
        &mut self,
        name: impl Into<String>,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        let declaration =
            WorthQueryWorkspaceLiveViewDeclaration::try_from_request(request, schema_view)?;
        let (request, schema_view) = declaration.into_parts();
        self.runtime.declare_live_view(name, request, schema_view)
    }

    pub fn computed<T>(
        &mut self,
        name: impl Into<String>,
        view: impl FnOnce(WorthQueryComputedBuilder) -> WorthQueryComputedBuilder,
        maintainer: impl WorthQueryDerivedViewMaintainer + 'static,
    ) -> Result<WorthQueryDerivedViewHandle<T>, WorthQueryRuntimeError> {
        let name = name.into();
        let view = view(WorthQueryComputedBuilder::surface(&name)).build()?;
        self.runtime
            .declare_maintained_derived_view(view, maintainer)
    }

    pub fn computed_view<T>(
        &mut self,
        view: WorthQueryDerivedView,
        maintainer: impl WorthQueryDerivedViewMaintainer + 'static,
    ) -> Result<WorthQueryDerivedViewHandle<T>, WorthQueryRuntimeError> {
        self.runtime
            .declare_maintained_derived_view(view, maintainer)
    }

    pub fn computed_definition(
        &mut self,
        name: impl Into<String>,
        view: impl FnOnce(WorthQueryComputedBuilder) -> WorthQueryComputedBuilder,
    ) -> Result<WorthQueryDerivedView, WorthQueryRuntimeError> {
        let name = name.into();
        let view = view(WorthQueryComputedBuilder::surface(&name)).build()?;
        self.runtime.declare_derived_view(view)
    }

    pub fn effect<T>(
        &mut self,
        name: impl Into<String>,
        declaration: impl FnOnce(WorthQueryEffectBuilder) -> WorthQueryEffectBuilder,
    ) -> Result<WorthQueryEffectHandle<T>, WorthQueryRuntimeError> {
        let name = name.into();
        let declaration = declaration(WorthQueryEffectBuilder::new(&name)).build()?;
        self.runtime.declare_effect(declaration)
    }

    pub fn effect_declaration<T>(
        &mut self,
        declaration: super::WorthQueryEffectDeclaration,
    ) -> Result<WorthQueryEffectHandle<T>, WorthQueryRuntimeError> {
        self.runtime.declare_effect(declaration)
    }

    pub fn preview<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.runtime.preview(label)
    }

    pub fn preview_with_options<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
        options: WorthQueryPreviewOptions,
    ) -> Result<WorthQueryPreviewSession<'a>, WorthQueryRuntimeError> {
        self.runtime.preview_with_options(label, options)
    }

    pub fn branch<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.runtime.branch(label)
    }

    pub fn branch_with_options<'a>(
        &'a mut self,
        label: WorthQuerySessionLabel,
        options: WorthQueryBranchOptions,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.runtime.branch_with_options(label, options)
    }

    pub fn intent(
        &mut self,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        self.runtime.execute_intent(declaration)
    }

    pub fn next_effect_intent<T>(
        &mut self,
        effect: &WorthQueryEffectHandle<T>,
        strategy_version: impl Into<String>,
        input_contract: impl Into<String>,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        self.runtime
            .execute_next_effect_write_intent(effect, strategy_version, input_contract)
    }
}
