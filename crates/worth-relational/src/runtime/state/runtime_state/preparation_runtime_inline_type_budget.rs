//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RelationalPreparationRuntime>(
            "crates/worth-relational/src/runtime/state/runtime_state/preparation_runtime.rs",
            "RelationalPreparationRuntime",
            128 * 1024,
        );
        owner
            .field::<crate::runtime::RelationalCheckpointManifestSelection>("checkpoint_selection");
        owner.field::<Option<crate::history::data::CanonicalIndexDefinitionDelta>>(
            "recovered_index_catalog",
        );
        owner.field::<Option<crate::history::data::CanonicalAuthoredIndexWrites>>(
            "recovered_authored_index_writes",
        );
        owner.field::<Arc<std::collections::BTreeSet<crate::symbols::data::Symbol>>>(
            "symbol_references",
        );
        owner.field::<Option<crate::execution::RequestWorkBudget>>("commit_work_budget");
        owner.field::<Arc<RelationalRuntimeConfig>>("config");
        owner.field::<Arc<SchemaContractRuntimeSubsystem>>("schema_contract_runtime");
        owner.field::<u64>("configuration_generation");
        owner.field::<Option<Arc<crate::runtime::RelationalRootExecutionContracts>>>(
            "execution_contracts",
        );
        owner
            .field::<Option<crate::branch::RetainedRelationalSchemaRegistry>>("execution_registry");
        owner.field::<Option<Arc<crate::runtime::AdmittedRelationalHomeProgramTarget>>>(
            "home_target",
        );
        owner.field::<crate::runtime::RelationalDiagnosticArtifactStore>("diagnostics");
        owner.field::<crate::runtime::state::IndexDefinitionReadBinding>("index_definitions");
        owner.field::<crate::runtime::MirrorResidencyDemandBinding>("mirror_residency");
        owner.field::<crate::durability::medium::RelationalDurablePreparationAdmission>(
            "durability_admission",
        );
        owner.field::<RelationalPreparationHistory>("history");
        owner.field::<RecordIdentitySubsystem>("record_identity");
        owner.field::<RuntimeServices>("services");
        owner.field::<RelationalRuntimeOwnerBinding>("lifecycle");
        owner.field::<RelationalRuntimePublicationBinding>("publication_owner");
        owner.field::<Arc<PublishedSnapshotCapacityOwner>>("snapshot_capacity");
        owner.field::<Arc<std::sync::atomic::AtomicU64>>("snapshot_ids");
        owner.field::<crate::runtime::LineageIdentityAllocator>("lineage_identity");
    }
}
