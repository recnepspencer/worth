use std::sync::Arc;

use crate::branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState};
use crate::performance::PerformanceAccess;
use crate::runtime::state::subsystems::visibility::SnapshotIdentityAllocator;
use crate::runtime::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotReservationDenial,
    PublishedSnapshotSlotReservation, RecordIdentitySubsystem, RelationalPreparationHistory,
    RelationalRuntimeConfig, RelationalRuntimeConfigurationBinding, RelationalRuntimeOwnerBinding,
    RelationalRuntimePublicationBinding, RuntimeServices, SchemaContractRuntimeSubsystem,
};

/// Cloneable owner binding. Configuration truth is captured per operation so
/// held ports observe atomic schema/configuration replacement.
#[derive(Debug, Clone)]
pub(crate) struct RelationalPreparationOwnerBinding {
    checkpoint_selection: crate::runtime::RelationalCheckpointManifestSelection,
    configuration: RelationalRuntimeConfigurationBinding,
    history: RelationalPreparationHistory,
    record_identity: RecordIdentitySubsystem,
    services: RuntimeServices,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication_owner: RelationalRuntimePublicationBinding,
    snapshot_capacity: Arc<PublishedSnapshotCapacityOwner>,
    snapshot_ids: Arc<SnapshotIdentityAllocator>,
    lineage_identity: crate::runtime::LineageIdentityAllocator,
    lineage_naming_basis: crate::runtime::state::subsystems::LineageNamingBasisBinding,
    diagnostics: crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore,
    index_definitions: super::indexing::RuntimeIndexDefinitionReadBinding,
    mirror_residency: crate::runtime::MirrorResidencyDemandBinding,
    durability_admission:
        crate::durability::published_medium::RelationalDurablePreparationAdmission,
}

mod source_inputs;

/// Narrow cloneable state used only for validation and candidate preparation.
#[derive(Debug, Clone)]
pub(crate) struct RelationalPreparationRuntime {
    pub(crate) checkpoint_selection: crate::runtime::RelationalCheckpointManifestSelection,
    pub(crate) current_ownership:
        Option<Arc<crate::publication::ownership::CurrentPublicationRoster>>,
    pub(crate) authored_coupling: Option<Arc<crate::mvcc::BoundAuthoredCoupling>>,
    pub(crate) symbol_references: Arc<std::collections::BTreeSet<crate::symbols::data::Symbol>>,
    pub(crate) commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    pub(crate) config: Arc<RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: Arc<SchemaContractRuntimeSubsystem>,
    pub(crate) configuration_generation: u64,
    pub(crate) configuration_epoch: super::RelationalRuntimeConfigurationSnapshot,
    pub(crate) execution_contracts: Option<Arc<crate::runtime::RelationalRootExecutionContracts>>,
    pub(crate) execution_registry: Option<crate::branch::RetainedRelationalSchemaRegistry>,
    pub(crate) home_target: Option<Arc<crate::runtime::AdmittedRelationalHomeProgramTarget>>,
    pub(crate) diagnostics: crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore,
    pub(crate) index_definitions: super::indexing::RuntimeIndexDefinitionReadBinding,
    pub(crate) mirror_residency: crate::runtime::MirrorResidencyDemandBinding,
    pub(crate) durability_admission:
        crate::durability::published_medium::RelationalDurablePreparationAdmission,
    pub(crate) history: RelationalPreparationHistory,
    pub(crate) record_identity: RecordIdentitySubsystem,
    pub(crate) services: RuntimeServices,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication_owner: RelationalRuntimePublicationBinding,
    snapshot_capacity: Arc<PublishedSnapshotCapacityOwner>,
    snapshot_ids: Arc<SnapshotIdentityAllocator>,
    pub(crate) lineage_identity: crate::runtime::LineageIdentityAllocator,
    pub(crate) lineage_naming_basis: crate::runtime::state::subsystems::LineageNamingBasisBinding,
}

impl RelationalPreparationOwnerBinding {
    pub(crate) fn from_runtime(runtime: &super::RelationalRuntime) -> Self {
        #[cfg(test)]
        crate::mvcc::split_contacts::binding_capture();
        Self {
            checkpoint_selection: runtime.durability.checkpoint_selection().clone(),
            configuration: runtime.configuration_binding(),
            history: crate::runtime::state::subsystems::history::history_preparation_binding::bind_preparation(&runtime.history),
            record_identity: runtime.record_identity.clone(),
            services: runtime.services.preparation_binding(),
            lifecycle: runtime.owner_binding(),
            publication_owner: runtime.publication_binding(),
            snapshot_capacity: runtime.visibility.published_snapshot_capacity_binding(),
            snapshot_ids: runtime.visibility.snapshot_identity_binding(),
            lineage_identity: runtime.lineage.identity_allocator(),
            lineage_naming_basis: runtime.lineage.naming_basis_binding(),
            diagnostics: runtime.publication.diagnostics.clone(),
            index_definitions: runtime.indexes.definition_read_binding(),
            durability_admission:
                crate::durability::published_medium::RelationalDurablePreparationAdmission::new(
                    runtime.durability.medium(),
                ),
            mirror_residency: runtime
                .partitions
                .residency_demand_binding(runtime.history.main_branch.clone()),
        }
    }

    pub(crate) fn configuration_binding(&self) -> RelationalRuntimeConfigurationBinding {
        self.configuration.clone()
    }

    pub(crate) fn runtime_snapshot(&self) -> RelationalPreparationRuntime {
        self.runtime_snapshot_from(&self.configuration.snapshot())
    }

    pub(crate) fn runtime_snapshot_from(
        &self,
        configuration: &crate::runtime::RelationalRuntimeConfigurationSnapshot,
    ) -> RelationalPreparationRuntime {
        RelationalPreparationRuntime {
            checkpoint_selection: self.checkpoint_selection.clone(),
            current_ownership: None,
            authored_coupling: None,
            symbol_references: Arc::default(),
            commit_work_budget: None,
            configuration_epoch: configuration.clone(),
            config: Arc::clone(&configuration.config),
            schema_contract_runtime: Arc::clone(&configuration.schema_contract_runtime),
            configuration_generation: configuration
                .schema_contract_runtime
                .custom_invariant_generation,
            execution_contracts: None,
            execution_registry: None,
            home_target: None,
            diagnostics: self.diagnostics.clone(),
            index_definitions: self.index_definitions.clone(),
            mirror_residency: self.mirror_residency.clone(),
            durability_admission: self.durability_admission.clone(),
            history: self.history.clone(),
            record_identity: self.record_identity.clone(),
            services: self.services.clone(),
            lifecycle: self.lifecycle.clone(),
            publication_owner: self.publication_owner.clone(),
            snapshot_capacity: Arc::clone(&self.snapshot_capacity),
            snapshot_ids: Arc::clone(&self.snapshot_ids),
            lineage_identity: self.lineage_identity.clone(),
            lineage_naming_basis: self.lineage_naming_basis.clone(),
        }
    }
}

impl RelationalPreparationRuntime {
    pub(crate) fn with_current_ownership(
        &self,
        current: Option<Arc<crate::publication::ownership::CurrentPublicationRoster>>,
    ) -> Self {
        let mut runtime = self.clone();
        runtime.current_ownership = current;
        runtime
    }
    pub(crate) fn with_authored_coupling(
        mut self,
        coupling: Option<Arc<crate::mvcc::BoundAuthoredCoupling>>,
    ) -> Self {
        self.authored_coupling = coupling;
        self
    }

    pub(crate) fn with_candidate_symbols(
        &self,
        symbols: &crate::runtime::PendingSymbolMeanings,
    ) -> Self {
        let mut runtime = self.clone();
        runtime.services.symbols = symbols.binding();
        runtime.symbol_references = Arc::new(symbols.required_references().clone());
        runtime
    }

    pub(crate) fn with_commit_work_budget(
        &self,
        prior: Option<crate::execution::RequestWorkBudget>,
    ) -> Self {
        let mut runtime = self.clone();
        if runtime.commit_work_budget.is_none() {
            runtime.commit_work_budget = Some(prior.unwrap_or_default());
        }
        runtime
    }

    pub(crate) fn admit_operation(
        &self,
    ) -> Option<super::owner_lifecycle::AdmittedRelationalRuntimeOperation> {
        #[cfg(test)]
        crate::mvcc::split_contacts::admission();
        self.lifecycle.admit()
    }

    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.services.runtime_instance_id()
    }

    pub(crate) fn current_version_id(&self) -> crate::identity::data::VersionId {
        self.history.current_version_id()
    }

    pub(crate) fn publication_binding(&self) -> RelationalRuntimePublicationBinding {
        self.publication_owner.clone()
    }

    pub(crate) fn reserve_published_snapshot_slot(
        &self,
    ) -> Result<PublishedSnapshotSlotReservation, PublishedSnapshotReservationDenial> {
        self.snapshot_capacity.reserve()
    }

    pub(crate) fn published_snapshot_count(&self) -> usize {
        self.snapshot_capacity.occupied_handles()
    }

    pub(crate) fn snapshot_id_allocator(&self) -> &SnapshotIdentityAllocator {
        &self.snapshot_ids
    }

    pub(crate) fn performance_access(&self) -> PerformanceAccess<'_> {
        PerformanceAccess::from_instrumentation(&self.services.instrumentation)
    }

    pub(crate) fn selected_branch_state(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<SelectedRelationalBranchState, crate::transactions::data::CommitPreparationError>
    {
        SelectedRelationalBranchState::from_owner_basis(self.runtime_instance_id(), basis)
    }

    pub(crate) fn push_bounded_preparation_diagnostic(
        &self,
        scope: crate::diagnostics::data::DiagnosticsScope,
        kind: crate::diagnostics::data::DiagnosticsArtifactKind,
        entries: Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
    ) -> crate::diagnostics::data::RelationalDiagnosticArtifact {
        self.preparation_diagnostics().emit(scope, kind, entries)
    }
}

impl crate::capabilities::RuntimeConfigSource for RelationalPreparationRuntime {
    fn runtime_config(&self) -> &RelationalRuntimeConfig {
        self.config.as_ref()
    }
}

impl crate::capabilities::InstrumentationSource for RelationalPreparationRuntime {
    fn runtime_instrumentation(&self) -> &crate::runtime::RuntimeInstrumentation {
        &self.services.instrumentation
    }
}

impl crate::capabilities::DiagnosticArtifactSink for RelationalPreparationRuntime {
    fn push_diagnostic_entries(
        &self,
        scope: crate::diagnostics::data::DiagnosticsScope,
        kind: crate::diagnostics::data::DiagnosticsArtifactKind,
        entries: Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
    ) {
        self.push_bounded_preparation_diagnostic(scope, kind, entries);
    }
}

impl crate::capabilities::SchemaSource for RelationalPreparationRuntime {
    fn schema_registry(&self) -> &crate::schema::data::RelationalSchemaRegistry {
        self.execution_registry
            .as_ref()
            .map_or(&self.config.schema.registry, |registry| registry.as_ref())
    }
}

impl crate::capabilities::AspectPlanSource for RelationalPreparationRuntime {
    fn aspect_plan_catalog(&self) -> &crate::schema::data::AspectContractPlanCatalog {
        &self.schema_contract_runtime.aspect_contract_plans
    }

    fn entity_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .entity_plans
            .get(&kind_id)
    }

    fn relation_aspect_plan(
        &self,
        kind_id: crate::identity::data::KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .relation_plans
            .get(&kind_id)
    }
}
#[cfg(test)]
#[path = "preparation_runtime_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[path = "preparation_runtime/working_state.rs"]
mod working_state;

#[path = "preparation_runtime/aggregate.rs"]
mod aggregate;

#[path = "preparation_runtime/schema_publication.rs"]
mod schema_publication;

mod applied_mutation;

mod candidate_registration;
mod history_preparation_inputs;
