//! Unit-only layout evidence; constructs no runtime, proof or authority.
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
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
        owner.field::<crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore>(
            "diagnostics",
        );
        owner
            .field::<crate::runtime::state::RuntimeIndexDefinitionReadBinding>("index_definitions");
        owner.field::<crate::runtime::MirrorResidencyDemandBinding>("mirror_residency");
        owner.field::<crate::durability::published_medium::RelationalDurablePreparationAdmission>(
            "durability_admission",
        );
        owner.field::<RelationalPreparationHistory>("history");
        owner.field::<RecordIdentitySubsystem>("record_identity");
        owner.field::<RuntimeServices>("services");
        owner.field::<RelationalRuntimeOwnerBinding>("lifecycle");
        owner.field::<RelationalRuntimePublicationBinding>("publication_owner");
        owner.field::<Arc<PublishedSnapshotCapacityOwner>>("snapshot_capacity");
        owner
            .field::<Arc<crate::runtime::state::subsystems::visibility::SnapshotIdentityAllocator>>(
                "snapshot_ids",
            );
        owner.field::<crate::runtime::LineageIdentityAllocator>("lineage_identity");
        owner.field::<crate::runtime::state::subsystems::LineageNamingBasisBinding>(
            "lineage_naming_basis",
        );
    }
}
