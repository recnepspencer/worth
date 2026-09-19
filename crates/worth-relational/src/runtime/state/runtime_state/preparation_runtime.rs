use std::sync::Arc;

use crate::branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState};
use crate::performance::PerformanceAccess;
use crate::runtime::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotSlotReservation, RecordIdentitySubsystem,
    RelationalPreparationHistory, RelationalRuntimeConfig, RelationalRuntimeConfigurationBinding,
    RelationalRuntimeOwnerBinding, RelationalRuntimePublicationBinding, RuntimeServices,
    SchemaContractRuntimeSubsystem,
};

/// Cloneable owner binding. Configuration truth is captured per operation so
/// held ports observe atomic schema/configuration replacement.
#[derive(Debug, Clone)]
pub(crate) struct RelationalPreparationOwnerBinding {
    configuration: RelationalRuntimeConfigurationBinding,
    history: RelationalPreparationHistory,
    record_identity: RecordIdentitySubsystem,
    services: RuntimeServices,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication_owner: RelationalRuntimePublicationBinding,
    snapshot_capacity: Arc<PublishedSnapshotCapacityOwner>,
    snapshot_ids: Arc<std::sync::atomic::AtomicU64>,
    lineage_identity: crate::runtime::LineageIdentityAllocator,
    diagnostics: crate::runtime::RelationalDiagnosticArtifactStore,
}

/// Narrow cloneable state used only for validation and candidate preparation.
#[derive(Debug, Clone)]
pub(crate) struct RelationalPreparationRuntime {
    pub(crate) config: Arc<RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: Arc<SchemaContractRuntimeSubsystem>,
    pub(crate) diagnostics: crate::runtime::RelationalDiagnosticArtifactStore,
    pub(crate) history: RelationalPreparationHistory,
    pub(crate) record_identity: RecordIdentitySubsystem,
    pub(crate) services: RuntimeServices,
    lifecycle: RelationalRuntimeOwnerBinding,
    publication_owner: RelationalRuntimePublicationBinding,
    snapshot_capacity: Arc<PublishedSnapshotCapacityOwner>,
    snapshot_ids: Arc<std::sync::atomic::AtomicU64>,
    pub(crate) lineage_identity: crate::runtime::LineageIdentityAllocator,
}

impl RelationalPreparationOwnerBinding {
    pub(crate) fn from_runtime(runtime: &super::RelationalRuntime) -> Self {
        Self {
            configuration: runtime.configuration_binding(),
            history: runtime.history.preparation_binding(),
            record_identity: runtime.record_identity.clone(),
            services: runtime.services.preparation_binding(),
            lifecycle: runtime.owner_binding(),
            publication_owner: runtime.publication_binding(),
            snapshot_capacity: runtime.visibility.published_snapshot_capacity_binding(),
            snapshot_ids: runtime.visibility.snapshot_identity_binding(),
            lineage_identity: runtime.lineage.identity_allocator(),
            diagnostics: runtime.publication.diagnostics.clone(),
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
            config: Arc::clone(&configuration.config),
            schema_contract_runtime: Arc::clone(&configuration.schema_contract_runtime),
            diagnostics: self.diagnostics.clone(),
            history: self.history.clone(),
            record_identity: self.record_identity.clone(),
            services: self.services.clone(),
            lifecycle: self.lifecycle.clone(),
            publication_owner: self.publication_owner.clone(),
            snapshot_capacity: Arc::clone(&self.snapshot_capacity),
            snapshot_ids: Arc::clone(&self.snapshot_ids),
            lineage_identity: self.lineage_identity.clone(),
        }
    }
}

impl RelationalPreparationRuntime {
    pub(crate) fn admit_operation(
        &self,
    ) -> Option<super::owner_lifecycle::AdmittedRelationalRuntimeOperation> {
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
    ) -> Result<PublishedSnapshotSlotReservation, usize> {
        self.snapshot_capacity.reserve()
    }

    pub(crate) fn published_snapshot_count(&self) -> usize {
        self.snapshot_capacity.occupied_handles()
    }

    pub(crate) fn allocate_snapshot_id(&self) -> Option<crate::snapshots::data::SnapshotId> {
        self.snapshot_ids
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
            .ok()
            .map(crate::snapshots::data::SnapshotId)
    }

    pub(crate) fn validate_reserved_lineage_events(
        &self,
        events: &[crate::lineage::data::LineageEventRecord],
    ) -> Result<(), String> {
        let mut previous = None;
        for event in events {
            if previous.is_some_and(|prior| event.event_id() <= prior) {
                return Err(
                    "live lineage event ids must advance within their reserved batch".to_owned(),
                );
            }
            if event.event_id() >= self.lineage_identity.frontiers().1 {
                return Err(format!(
                    "live lineage event {} was not reserved by this allocator",
                    event.event_id()
                ));
            }
            previous = Some(event.event_id());
        }
        Ok(())
    }

    pub(crate) fn performance_access(&self) -> PerformanceAccess<'_> {
        PerformanceAccess::from_instrumentation(&self.services.instrumentation)
    }

    pub(crate) fn selected_branch_state(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<SelectedRelationalBranchState, crate::transactions::data::CommitPreparationError>
    {
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(crate::transactions::data::CommitPreparationError::selected_branch_root_reference_mismatch(
                basis.identity().branch_id().clone(),
                basis.observation().commit_id().map(|id| id.0),
                basis.observation().version_id(),
            ));
        }
        Ok(SelectedRelationalBranchState::from_admitted_basis(basis))
    }

    pub(crate) fn admitted_branch_basis_version(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Option<crate::identity::data::VersionId> {
        let cell = self.history.branch_cell(basis.identity().branch_id())?;
        if cell.identity() != basis.identity()
            || cell.observation() != *basis.reference()
            || cell.truth_version() != basis.truth_version()
        {
            return None;
        }
        match basis.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => {
                Some(crate::identity::data::VersionId(0))
            }
            worth_foundational::FoundationalBranchTarget::Basis(target) => cell
                .root()
                .and_then(|root| root.canonical_envelope().cloned())
                .filter(|envelope| envelope.commit.commit_id.0 == target.selected_commit_id())
                .map(|envelope| envelope.commit.version_id),
        }
    }

    pub(crate) fn push_bounded_preparation_diagnostic(
        &self,
        scope: crate::diagnostics::data::DiagnosticsScope,
        kind: crate::diagnostics::data::DiagnosticsArtifactKind,
        entries: Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
    ) -> crate::diagnostics::data::RelationalDiagnosticArtifact {
        let artifact = crate::diagnostics::data::RelationalDiagnosticArtifact::new(
            scope,
            kind,
            crate::diagnostics::data::DeterminismExpectation::Required,
            entries,
        );
        let filtered = self
            .config
            .diagnostics
            .profile
            .filter_artifact(artifact.clone())
            .unwrap_or_else(|| {
                crate::diagnostics::data::RelationalDiagnosticArtifact::new(
                    artifact.scope,
                    artifact.kind,
                    artifact.determinism,
                    Vec::new(),
                )
            });
        if !filtered.entries.is_empty() {
            self.diagnostics.push(filtered.clone());
        }
        filtered
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

impl crate::capabilities::PublicationPolicySource for RelationalPreparationRuntime {
    fn max_patch_records_per_commit(&self) -> usize {
        self.config.publication.policy.max_patch_records_per_commit
    }
}

impl crate::capabilities::SchemaSource for RelationalPreparationRuntime {
    fn schema_registry(&self) -> &crate::schema::data::RelationalSchemaRegistry {
        &self.config.schema.registry
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
