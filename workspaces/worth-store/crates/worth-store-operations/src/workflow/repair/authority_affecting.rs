use std::path::{Path, PathBuf};

use super::integrity_classification::{
    IntegrityOperationalRepairOwner, IntegrityRepairClassificationDenial,
    IntegrityRepairClassificationPlan,
};
use crate::workflow::restore::{
    BackupRestoreReplayDenial, BackupRestoreReplayOwner, BackupRestoreReplayPlan,
    BackupRestoreReplayRequest,
};
use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_backend::{
    LoweredNonCurrentStagingPlan, NonCurrentStagingLoweringDenial, NonCurrentStagingPlanRequest,
    PhysicalRecoveryStagingOwner,
};

use crate::authorization::{
    authorize_lowered_plan, consume_authorization_through, recover_authorization_consumption,
    AuthorizationReplayPolicy, AuthorizedOperationalPlan, LoweredOperationalPlan,
};
use crate::owner_plan_dag::{DestructiveOperationKind, OperationalPlanBinding};
use crate::{
    AuthorizationDenial, AuthorizationRevocationObservation, ExternalOperatorAssertion,
    IndeterminateRepairRecoveryHandle, OperationalAuthorizationPort, OperationalControlStore,
    OperationalOperationId, OperationalSecurityScope, OperationalTransitionId,
    ProductionRestoreAdmissibleBackupBundle,
};

use super::authority_owner_dag::{repair_dag, RepairOwnerNodes};
use super::authority_staging_artifacts::{operation_identity, path_identity, staging_artifacts};
use super::journal::RepairExecutionJournal;
use super::{
    AuthorityAffectingRepairOperation, AuthorityAffectingRepairReadinessDenial,
    ExecutionReadyAuthorityAffectingRepair, RepairCandidateSet, RepairJournalDenial,
};

#[derive(Debug)]
pub struct AuthorityAffectingStagedRepairPlan {
    operation_id: OperationalOperationId,
    damaged: Vec<super::resolved_region::ResolvedRepairRegion>,
    basis_identity: [u8; 32],
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    security_scope: OperationalSecurityScope,
    pub(super) backup: ProductionRestoreAdmissibleBackupBundle,
    target_parent: PathBuf,
    admitted_capacity_bytes: u64,
    copy_buffer_bytes: usize,
}

impl AuthorityAffectingStagedRepairPlan {
    pub(super) fn from_resolved(
        candidates: RepairCandidateSet,
        backup: ProductionRestoreAdmissibleBackupBundle,
        target_parent: PathBuf,
        admitted_capacity_bytes: u64,
        copy_buffer_bytes: usize,
    ) -> Self {
        Self {
            operation_id: candidates.operation_id,
            damaged: candidates.damaged,
            basis_identity: candidates.basis_identity,
            authority_identity: candidates.authority_identity,
            security_scope: candidates.security_scope,
            backup,
            target_parent,
            admitted_capacity_bytes,
            copy_buffer_bytes,
        }
    }

    pub fn lower_owners(
        self,
    ) -> Result<LoweredAuthorityAffectingRepairOwnerPlanDag, AuthorityAffectingRepairLoweringDenial>
    {
        let artifacts = staging_artifacts(&self)?;
        let source_root = self.source_root().to_path_buf();
        let integrity_regions = super::region_projection::integrity_regions(&self.damaged)
            .map_err(|_| AuthorityAffectingRepairLoweringDenial::AllocationFailed)?;
        let integrity = IntegrityOperationalRepairOwner::lower(integrity_regions)
            .map_err(AuthorityAffectingRepairLoweringDenial::Integrity)?;
        let backend = PhysicalRecoveryStagingOwner::lower(NonCurrentStagingPlanRequest::new(
            operation_identity(&self.operation_id),
            source_root,
            &self.target_parent,
            artifacts,
            self.admitted_capacity_bytes,
            self.copy_buffer_bytes,
        ))
        .map_err(AuthorityAffectingRepairLoweringDenial::Backend)?;
        let manifest = self.backup.custody().structural().materialized().manifest();
        let source_identity = self
            .backup
            .custody()
            .structural()
            .materialized()
            .manifest_digest();
        let recovery =
            BackupRestoreReplayOwner::lower(BackupRestoreReplayRequest::from_verified_backup(
                manifest,
                source_identity,
                backend.binding(),
            ))
            .map_err(AuthorityAffectingRepairLoweringDenial::Recovery)?;
        let layout_regions = super::region_projection::layout_repair_regions(integrity.regions())?;
        let layout = worth_store_layout_indexes::LayoutRepairConsequenceOwner::lower(
            &layout_regions,
            backend.binding(),
        )
        .map_err(AuthorityAffectingRepairLoweringDenial::Layout)?;
        let blob_regions = super::region_projection::blob_repair_regions(integrity.regions())?;
        let blob = worth_store_blob_chunks::BlobRepairConsequenceOwner::lower(
            &blob_regions,
            backend.binding(),
        )
        .map_err(AuthorityAffectingRepairLoweringDenial::Blob)?;
        let (dag, nodes) = repair_dag(
            &integrity,
            &backend,
            &recovery,
            layout.as_ref(),
            blob.as_ref(),
        )?;
        let explanation = dag.explanation().clone();
        let target_identity = path_identity(&self.target_parent);
        let binding = OperationalPlanBinding::bind(
            DestructiveOperationKind::AuthorityAffectingRepair,
            dag,
            self.authority_identity,
            self.security_scope,
            self.basis_identity,
            target_identity,
            recovery.fingerprint(),
        );
        Ok(LoweredAuthorityAffectingRepairOwnerPlanDag {
            operation_id: self.operation_id,
            authorization: LoweredOperationalPlan::from_binding(binding),
            integrity,
            backend,
            recovery,
            layout,
            blob,
            nodes,
            explanation,
        })
    }

    fn source_root(&self) -> &Path {
        self.backup.custody().structural().materialized().root()
    }
}

#[derive(Debug)]
pub enum AuthorityAffectingRepairLoweringDenial {
    AllocationFailed,
    Integrity(IntegrityRepairClassificationDenial),
    SourceArtifactUnavailable { output_name: String },
    InvalidSourceArtifact { output_name: String },
    Backend(NonCurrentStagingLoweringDenial),
    Recovery(BackupRestoreReplayDenial),
    Layout(worth_store_layout_indexes::LayoutRepairConsequenceDenial),
    Blob(worth_store_blob_chunks::BlobRepairConsequenceDenial),
    OwnerDag(crate::OwnerPlanDagDenial),
    InvalidFootprint,
    CounterOverflow,
}

#[derive(Debug, Clone)]
pub struct LoweredAuthorityAffectingRepairOwnerPlanDag {
    operation_id: OperationalOperationId,
    authorization: LoweredOperationalPlan<AuthorityAffectingRepairOperation>,
    integrity: IntegrityRepairClassificationPlan,
    backend: LoweredNonCurrentStagingPlan,
    recovery: BackupRestoreReplayPlan,
    layout: Option<worth_store_layout_indexes::LayoutRepairConsequencePlan>,
    blob: Option<worth_store_blob_chunks::BlobRepairConsequencePlan>,
    nodes: RepairOwnerNodes,
    explanation: crate::CanonicalOwnerPlanDagExplanation,
}

#[derive(Debug)]
pub struct AuthorizedAuthorityAffectingRepairPlan {
    operation_id: OperationalOperationId,
    authorization: AuthorizedOperationalPlan<AuthorityAffectingRepairOperation>,
    integrity: IntegrityRepairClassificationPlan,
    backend: LoweredNonCurrentStagingPlan,
    recovery: BackupRestoreReplayPlan,
    layout: Option<worth_store_layout_indexes::LayoutRepairConsequencePlan>,
    blob: Option<worth_store_blob_chunks::BlobRepairConsequencePlan>,
    nodes: RepairOwnerNodes,
}

impl LoweredAuthorityAffectingRepairOwnerPlanDag {
    pub const fn explanation(&self) -> &crate::CanonicalOwnerPlanDagExplanation {
        &self.explanation
    }

    #[allow(clippy::too_many_arguments)]
    pub fn authorize(
        self,
        port: &impl OperationalAuthorizationPort,
        assertion: &ExternalOperatorAssertion,
        requested_at: u64,
        expires_at: u64,
        replay_policy: AuthorizationReplayPolicy,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<AuthorizedAuthorityAffectingRepairPlan, AuthorizationDenial> {
        Ok(AuthorizedAuthorityAffectingRepairPlan {
            operation_id: self.operation_id,
            authorization: authorize_lowered_plan(
                self.authorization,
                port,
                assertion,
                requested_at,
                expires_at,
                replay_policy,
                revocation,
            )?,
            integrity: self.integrity,
            backend: self.backend,
            recovery: self.recovery,
            layout: self.layout,
            blob: self.blob,
            nodes: self.nodes,
        })
    }
}

impl AuthorizedAuthorityAffectingRepairPlan {
    pub fn ready<'a>(
        self,
        control: &'a OperationalControlStore,
        transition: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyAuthorityAffectingRepair<'a>, AuthorityAffectingRepairReadinessDenial>
    {
        self.ready_through(
            control,
            control,
            transition,
            current,
            observed_at,
            revocation,
        )
    }

    pub(super) fn ready_through<'a>(
        self,
        control: &'a OperationalControlStore,
        append: &'a dyn crate::OperationalControlStorePort,
        transition: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyAuthorityAffectingRepair<'a>, AuthorityAffectingRepairReadinessDenial>
    {
        if self.authorization.binding().authority_identity() != current.authority_identity() {
            return Err(AuthorityAffectingRepairReadinessDenial::StaleAuthority);
        }
        let target_parent = self.backend.binding().staging_root().parent().ok_or(
            AuthorityAffectingRepairReadinessDenial::Target(
                crate::control_store::NonCurrentRecoveryTargetDenial::Unavailable,
            ),
        )?;
        let target_admission = control
            .admit_non_current_recovery_target(target_parent)
            .map_err(AuthorityAffectingRepairReadinessDenial::Target)?;
        let operation_id = self.operation_id;
        let staging_authority = self.authorization.binding().authority_identity();
        let security_scope = self.authorization.binding().security_scope();
        let consumed = consume_authorization_through(
            control,
            append,
            operation_id.clone(),
            transition,
            self.authorization,
            None,
            observed_at,
            revocation,
        )
        .map_err(AuthorityAffectingRepairReadinessDenial::Authorization)?;
        let journal = RepairExecutionJournal::open_through(
            control,
            append,
            current.authority_identity(),
            operation_id.clone(),
            consumed.receipt().authorization_identity(),
            consumed.authorized().binding().fingerprint(),
            self.nodes.count(),
            crate::RepairRecoveryTopology::NonCurrentAuthorityAffecting,
        )
        .map_err(AuthorityAffectingRepairReadinessDenial::Journal)?;
        Ok(ExecutionReadyAuthorityAffectingRepair {
            operation_id,
            authorization: consumed.receipt(),
            staging_authority,
            security_scope,
            integrity: self.integrity,
            backend: self.backend,
            recovery: self.recovery,
            layout: self.layout,
            blob: self.blob,
            nodes: self.nodes,
            journal,
            _target_admission: target_admission,
        })
    }
}

impl LoweredAuthorityAffectingRepairOwnerPlanDag {
    pub fn recover_ready<'a>(
        self,
        handle: &IndeterminateRepairRecoveryHandle,
        control: &'a OperationalControlStore,
        current: &StoreCurrentAuthorityWitness,
    ) -> Result<ExecutionReadyAuthorityAffectingRepair<'a>, AuthorityAffectingRepairReadinessDenial>
    {
        self.recover_ready_through(handle, control, control, current)
    }

    pub(super) fn recover_ready_through<'a>(
        self,
        handle: &IndeterminateRepairRecoveryHandle,
        control: &'a OperationalControlStore,
        append: &'a dyn crate::OperationalControlStorePort,
        current: &StoreCurrentAuthorityWitness,
    ) -> Result<ExecutionReadyAuthorityAffectingRepair<'a>, AuthorityAffectingRepairReadinessDenial>
    {
        let binding = self.authorization.binding();
        if binding.authority_identity() != current.authority_identity()
            || self.operation_id != *handle.operation_id()
            || binding.fingerprint() != handle.plan_fingerprint()
        {
            return Err(AuthorityAffectingRepairReadinessDenial::StaleAuthority);
        }
        if !self
            .nodes
            .admits_recovered_receipts(handle.durable_owner_receipts())
            || !self
                .nodes
                .admits_recovered_starts(handle.started_owner_nodes())
        {
            return Err(AuthorityAffectingRepairReadinessDenial::Journal(
                RepairJournalDenial::InvalidHistory,
            ));
        }
        let target_parent = self.backend.binding().staging_root().parent().ok_or(
            AuthorityAffectingRepairReadinessDenial::Target(
                crate::control_store::NonCurrentRecoveryTargetDenial::Unavailable,
            ),
        )?;
        let target_admission = control
            .admit_non_current_recovery_target(target_parent)
            .map_err(AuthorityAffectingRepairReadinessDenial::Target)?;
        let authorization = recover_authorization_consumption(
            control,
            handle.operation_id(),
            handle.authorization_identity(),
            handle.plan_fingerprint(),
        )
        .map_err(AuthorityAffectingRepairReadinessDenial::Authorization)?;
        let journal = RepairExecutionJournal::recover_through(
            control,
            append,
            current.authority_identity(),
            handle,
        )
        .map_err(AuthorityAffectingRepairReadinessDenial::Journal)?;
        Ok(ExecutionReadyAuthorityAffectingRepair {
            operation_id: self.operation_id,
            authorization,
            staging_authority: binding.authority_identity(),
            security_scope: binding.security_scope(),
            integrity: self.integrity,
            backend: self.backend,
            recovery: self.recovery,
            layout: self.layout,
            blob: self.blob,
            nodes: self.nodes,
            journal,
            _target_admission: target_admission,
        })
    }
}
