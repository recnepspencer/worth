use std::path::{Path, PathBuf};

use worth_store_authority::{BackupRestoreAdmissionPolicy, StoreCurrentAuthorityWitness};
use worth_store_offline_verifier::{verify_materialized_backup, OfflineInspectionBudget};
use worth_store_physical_isolation::{
    BackupCutCoordinates, BackupCutManifest, BackupReachabilityLeaseRegistry,
};
use worth_store_security::StoreKeyVersionPosture;

use crate::{
    admit_backup_for_production_restore, qualify_backup_custody,
    record_independent_backup_verification, BackupExportCustodyDeclaration,
    BackupExportCustodyMode, BackupExportCustodyReadiness, ConfiguredFailureDomainId,
    OnlineBackupIntent, OperationalControlLocation, OperationalControlStore,
    OperationalControlStorePort, OperationalCounterReceipt, OperationalOperationId,
    ProductionRestoreAdmissibleBackupBundle, ProtectedOperationalMediaLocation,
};

use super::backup_artifacts::{
    canonical_backup_artifacts_with_blob_count, CanonicalBackupArtifacts,
};
use super::poisoned_backup::{reject_poisoned_backup, RejectedPoisonedBackupScenario};

pub struct OwnerBackedBackupScenario {
    workspace: ScenarioWorkspace,
    source: PathBuf,
    target: PathBuf,
    control: PathBuf,
    artifacts: CanonicalBackupArtifacts,
    leases: BackupReachabilityLeaseRegistry,
    authority: StoreCurrentAuthorityWitness,
}

enum ScenarioWorkspace {
    Temporary(tempfile::TempDir),
    External(PathBuf),
}

impl ScenarioWorkspace {
    fn path(&self) -> &Path {
        match self {
            Self::Temporary(directory) => directory.path(),
            Self::External(path) => path,
        }
    }
}

pub struct OwnerBackedBackupOutcome {
    operation: OperationalOperationId,
    restore_source: ProductionRestoreAdmissibleBackupBundle,
    counters: OperationalCounterReceipt,
}

pub struct OwnerBackedBackupAbandonmentOutcome {
    operation: OperationalOperationId,
    receipt: worth_store_physical_isolation::BackupCutAbandonmentReceipt,
    counters: OperationalCounterReceipt,
}

impl OwnerBackedBackupScenario {
    pub fn materialize(case: &str) -> Self {
        Self::materialize_with_blob_count(case, 1)
    }

    pub fn materialize_with_blob_count(case: &str, blob_count: u64) -> Self {
        let workspace = tempfile::tempdir().expect("certification scenario workspace");
        Self::materialize_in_workspace(case, blob_count, ScenarioWorkspace::Temporary(workspace))
    }

    pub fn materialize_at(case: &str, blob_count: u64, workspace: impl Into<PathBuf>) -> Self {
        Self::materialize_in_workspace(
            case,
            blob_count,
            ScenarioWorkspace::External(workspace.into()),
        )
    }

    fn materialize_in_workspace(case: &str, blob_count: u64, workspace: ScenarioWorkspace) -> Self {
        let source = workspace.path().join("backup-source");
        let target = workspace.path().join("backup-target");
        let control = workspace.path().join("control/operations.log");
        std::fs::create_dir_all(&source).expect("backup source directory");
        std::fs::create_dir_all(&target).expect("backup target directory");
        let authority = crate::backup::export::current_authority(case);
        let store_identity =
            worth_store_physical_format::PhysicalStoreIdentity::from_aspect_identity(
                authority.identity().clone(),
            );
        let artifacts = canonical_backup_artifacts_with_blob_count(
            case,
            &source,
            1,
            blob_count,
            store_identity,
        );
        Self {
            artifacts,
            leases: BackupReachabilityLeaseRegistry::for_store_runtime(),
            authority,
            workspace,
            source,
            target,
            control,
        }
    }

    pub fn control_store(&self) -> OperationalControlStore {
        OperationalControlStore::open_with_certified_topology(
            OperationalControlLocation::new(&self.control, domain("control")),
            [
                ProtectedOperationalMediaLocation::source(&self.source, domain("source")),
                ProtectedOperationalMediaLocation::backup_target(&self.target, domain("target")),
            ],
        )
        .expect("physically independent certification control store")
    }

    pub fn independent_control_store_at(
        &self,
        path: impl Into<PathBuf>,
        failure_domain_id: &str,
    ) -> OperationalControlStore {
        OperationalControlStore::open_with_certified_topology(
            OperationalControlLocation::new(path, domain(failure_domain_id)),
            [
                ProtectedOperationalMediaLocation::source(&self.source, domain("source")),
                ProtectedOperationalMediaLocation::backup_target(&self.target, domain("target")),
            ],
        )
        .expect("physically independent certification control store")
    }

    pub const fn authority(&self) -> &StoreCurrentAuthorityWitness {
        &self.authority
    }

    pub fn workspace_root(&self) -> &Path {
        self.workspace.path()
    }

    pub fn source_root(&self) -> &Path {
        &self.source
    }

    pub fn security_scope_identity(&self) -> worth_store_security::StoreSecurityScopeIdentity {
        backup_custody(&self.authority).identity()
    }

    pub fn operational_security_scope(&self) -> crate::OperationalSecurityScope {
        crate::OperationalSecurityScope::from_admission(backup_custody(&self.authority).receipt())
    }

    pub fn execute(
        &self,
        scenario_identity: &str,
        control: &impl OperationalControlStorePort,
    ) -> OwnerBackedBackupOutcome {
        self.execute_named(scenario_identity, "backup", control)
    }

    pub fn execute_named(
        &self,
        scenario_identity: &str,
        operation_label: &str,
        control: &impl OperationalControlStorePort,
    ) -> OwnerBackedBackupOutcome {
        let (operation, admitted) = self.admit(scenario_identity, operation_label, control);
        let completion = admitted
            .materialize(self.operation_target(&operation), 64 * 1024, control)
            .expect("owner-opened backup materialization")
            .finish()
            .expect("owner-completed backup materialization");
        let counters = OperationalCounterReceipt::from_backup_materialization(&completion)
            .expect("bounded backup counters");
        let (materialized, cut) = completion.into_parts();
        let structural = verify_materialized_backup(
            materialized,
            OfflineInspectionBudget::bounded(64 * 1024, u64::MAX)
                .expect("bounded backup verification"),
        )
        .expect("independent structural backup verification");
        let verified = record_independent_backup_verification(
            &operation,
            structural,
            cut,
            control,
            &self.leases,
        )
        .expect("durable independent verification and lease release");
        let qualified = qualify_backup_custody(verified, &backup_custody(&self.authority))
            .expect("custody-qualified backup");
        let restore_source = admit_backup_for_production_restore(
            qualified,
            &self.authority,
            BackupRestoreAdmissionPolicy::production_default(),
        )
        .expect("production restore admission");
        OwnerBackedBackupOutcome {
            operation,
            restore_source,
            counters,
        }
    }

    pub fn abandon(
        &self,
        scenario_identity: &str,
        control: &impl OperationalControlStorePort,
    ) -> OwnerBackedBackupAbandonmentOutcome {
        let (operation, admitted) = self.admit(scenario_identity, "backup-abandonment", control);
        let receipt = admitted
            .abandon("certified interrupted backup", control, &self.leases)
            .expect("durable owner-backed backup abandonment");
        let counters = OperationalCounterReceipt::from_backup_abandonment(&operation, &receipt);
        OwnerBackedBackupAbandonmentOutcome {
            operation,
            receipt,
            counters,
        }
    }

    pub fn reject_poisoned_materialization(
        &self,
        scenario_identity: &str,
        control: &impl OperationalControlStorePort,
    ) -> RejectedPoisonedBackupScenario {
        let (operation, admitted) = self.admit(scenario_identity, "poisoned-backup", control);
        let target = self.operation_target(&operation);
        reject_poisoned_backup(operation, admitted, &target, control, &self.leases)
    }

    fn admit(
        &self,
        scenario_identity: &str,
        operation_label: &str,
        control: &impl OperationalControlStorePort,
    ) -> (OperationalOperationId, crate::AdmittedOnlineBackup) {
        let operation =
            OperationalOperationId::new(format!("{scenario_identity}/{operation_label}"))
                .expect("scenario operation identity");
        let custody = backup_custody(&self.authority);
        let admitted = OnlineBackupIntent::new(
            operation.clone(),
            self.coordinates(),
            BackupCutManifest::canonical(self.artifacts.references.clone())
                .expect("owner-built complete backup cut"),
            custody,
        )
        .admit_cut(&self.authority, control, &self.leases)
        .expect("owner-admitted backup cut");
        (operation, admitted)
    }

    fn coordinates(&self) -> BackupCutCoordinates {
        BackupCutCoordinates::new(
            "lineage/s10-certification",
            1,
            1,
            &self.artifacts.checkpoint_identity,
            10,
            10,
            12,
            12,
            "worth-physical-format-v1",
            "posix-file-fsync-dir-sync",
        )
        .expect("coherent certification backup coordinates")
    }

    fn operation_target(&self, operation: &OperationalOperationId) -> PathBuf {
        use std::fmt::Write;

        let mut directory = String::from("operation-");
        for byte in &operation.stable_fingerprint()[..16] {
            write!(directory, "{byte:02x}").expect("writing to String cannot fail");
        }
        self.target.join(directory)
    }
}

pub fn reopen_owner_backed_control_store_at(workspace: &Path) -> OperationalControlStore {
    let source = workspace.join("backup-source");
    let target = workspace.join("backup-target");
    OperationalControlStore::open_with_certified_topology(
        OperationalControlLocation::new(
            workspace.join("control/operations.log"),
            domain("control"),
        ),
        [
            ProtectedOperationalMediaLocation::source(source, domain("source")),
            ProtectedOperationalMediaLocation::backup_target(target, domain("target")),
        ],
    )
    .expect("reopen certification control store")
}

impl OwnerBackedBackupOutcome {
    pub const fn operation(&self) -> &OperationalOperationId {
        &self.operation
    }

    pub const fn restore_source(&self) -> &ProductionRestoreAdmissibleBackupBundle {
        &self.restore_source
    }

    pub const fn counters(&self) -> OperationalCounterReceipt {
        self.counters
    }

    pub fn into_restore_source(self) -> ProductionRestoreAdmissibleBackupBundle {
        self.restore_source
    }
}

impl OwnerBackedBackupAbandonmentOutcome {
    pub const fn operation(&self) -> &OperationalOperationId {
        &self.operation
    }

    pub const fn receipt(&self) -> &worth_store_physical_isolation::BackupCutAbandonmentReceipt {
        &self.receipt
    }

    pub const fn counters(&self) -> OperationalCounterReceipt {
        self.counters
    }
}

fn backup_custody(authority: &StoreCurrentAuthorityWitness) -> BackupExportCustodyReadiness {
    let admission = BackupExportCustodyDeclaration::native(
        authority,
        BackupExportCustodyMode::Backup,
        StoreKeyVersionPosture::Current,
    )
    .expect("backup custody declaration")
    .admit_with_current_authority(authority)
    .expect("backup custody admission");
    BackupExportCustodyReadiness::from_admitted_custody(admission)
        .expect("backup custody readiness")
}

fn domain(label: &str) -> ConfiguredFailureDomainId {
    ConfiguredFailureDomainId::new(label).expect("certification failure domain")
}

#[cfg(test)]
mod tests {
    use crate::{OperationalSessionDisposition, OperationalSessionKind};

    use super::OwnerBackedBackupScenario;

    #[test]
    fn owner_backed_backup_completion_and_abandonment_are_distinct_valid_sessions() {
        let scenario = OwnerBackedBackupScenario::materialize("backup-session-dispositions");
        let control = scenario.control_store();

        let completed = scenario.execute("backup-session-dispositions", &control);
        let abandoned = scenario.abandon("backup-session-dispositions", &control);

        assert_eq!(
            completed.counters().disposition(),
            OperationalSessionDisposition::Completed
        );
        assert_eq!(
            abandoned.counters().disposition(),
            OperationalSessionDisposition::Abandoned
        );
        completed.counters().validate_structure().unwrap();
        abandoned.counters().validate_structure().unwrap();
        assert_ne!(completed.operation(), abandoned.operation());
        assert!(!abandoned.receipt().reason().is_empty());
        assert!(
            scenario
                .target
                .read_dir()
                .expect("materialized backup target")
                .next()
                .is_some(),
            "completed owner-backed backup must mutate its declared target"
        );
    }

    #[test]
    fn poisoned_backup_rejection_keeps_the_source_cut_reachable() {
        let scenario = OwnerBackedBackupScenario::materialize("poisoned-backup-retention");
        let control = scenario.control_store();

        let rejected =
            scenario.reject_poisoned_materialization("poisoned-backup-retention", &control);

        assert!(!rejected.omitted_artifact().is_empty());
        assert!(!rejected.torn_wal_artifact().is_empty());
        assert!(!rejected.substituted_index_artifact().is_empty());
        assert!(rejected.independently_localized_defects() >= 3);
        assert_ne!(rejected.rejection_identity(), [0; 32]);
        assert!(rejected.retained_source_leases() > 0);
        assert_eq!(rejected.counters().kind(), OperationalSessionKind::Backup);
        rejected.counters().validate_structure().unwrap();
    }
}
