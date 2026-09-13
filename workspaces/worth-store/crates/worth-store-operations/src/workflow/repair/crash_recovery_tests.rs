use std::cell::Cell;

use super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairOwnerBinding, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};
use sha2::{Digest, Sha256};
use worth_store_layout_indexes::DerivedIndexRepairRequest;

use super::intent::physical_target_identity;
use super::RepairCandidateSet;
use crate::phase_1_6_tests::support::{backup_custody, BackupScenario};
use crate::phase_7_13_tests::{operator_assertion, ExactAuthorizationPort, ExactControlSelection};
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalSecurityScope, OperationalTransitionId, OwnerPlanNodeIdentity,
    RepairExecutionBoundary, RepairExecutionBoundaryMoment, RepairExecutionControlPort,
    RepairExecutionInterrupted, RepairRecoveryDisposition, RepairRecoveryDispositionDenial,
    StoreOwnerKind,
};

#[test]
fn crash_after_current_owner_effect_requires_exact_resume_instead_of_false_abandonment() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("layout.index");
    let replacement = directory.path().join("layout.rebuilt");
    std::fs::write(&target, b"damaged layout").unwrap();
    std::fs::write(&replacement, b"rebuilt layout").unwrap();
    let authority = crate::backup::export::current_authority("repair-effect-crash");
    let lowered = lowered_repair(&authority, &target, &replacement);
    let restart = lowered.clone();
    let layout_node = lowered
        .explanation()
        .nodes()
        .iter()
        .find(|node| node.owner() == StoreOwnerKind::LayoutIndexes)
        .expect("layout owner node")
        .identity();
    let scenario = BackupScenario::new("repair-effect-crash-control");
    let control = scenario.control_store();
    let ready = lowered
        .authorize(
            &ExactAuthorizationPort {
                substitute_plan: None,
            },
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &control,
            OperationalTransitionId::new("repair-effect-crash-ready").unwrap(),
            &authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap();
    let interruption = ready
        .execute_with_control(&InterruptAt::once(
            layout_node,
            RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
        ))
        .expect_err("process loss lands after durable owner effect");
    assert!(matches!(
        interruption,
        super::RepairExecutionDenial::Interrupted(_)
    ));
    assert_eq!(std::fs::read(&target).unwrap(), b"rebuilt layout");

    let handle = repair_handle(&authority, &control);
    assert!(handle
        .started_owner_nodes()
        .iter()
        .any(|started| started.node_fingerprint() == layout_node.fingerprint()));
    assert!(!handle
        .durable_owner_receipts()
        .iter()
        .any(|receipt| receipt.node_fingerprint() == layout_node.fingerprint()));
    assert_eq!(
        handle.recovery_disposition(),
        RepairRecoveryDisposition::CurrentAuthorityResumeRequired {
            durable_owner_effects: 1,
        }
    );
    assert!(matches!(
        handle.abandon_before_mutation(&control, &authority, [0x71; 32]),
        Err(RepairRecoveryDispositionDenial::MutationAlreadyRequiresResume)
    ));
    restart
        .recover_ready(&handle, &control, &authority)
        .expect("exact plan and owner starts rebind")
        .execute()
        .expect("owner validates the already durable replacement and converges");
    assert!(repair_handles(&authority, &control).is_empty());
}

#[test]
fn crash_after_non_current_backend_effect_can_retain_only_explicit_isolated_residue() {
    let world = crate::phase_7_13_tests::restore_world("repair-isolated-crash");
    let manifest = world
        .admissible
        .custody()
        .structural()
        .materialized()
        .manifest();
    let row = manifest
        .artifacts()
        .iter()
        .find(|row| row.family() == worth_store_physical_format::BackupBundleArtifactFamily::Index)
        .unwrap();
    let source = world.backup_root.join(row.output_name());
    let candidates = RepairCandidateSet {
        operation_id: OperationalOperationId::new("repair-isolated-crash").unwrap(),
        damaged: vec![super::resolved_region::ResolvedRepairRegion::new(
            IntegrityRepairRegion::bounded(
                [0x81; 32],
                0,
                row.bytes(),
                IntegrityRepairRegionClass::QuarantineRequired,
                row.content_digest(),
                physical_target_identity(&source).unwrap(),
                IntegrityRepairOwnerBinding::observed(
                    IntegrityRepairArtifactFamily::LayoutIndex,
                    Some(row.generation()),
                    row.reclaim_owner()
                        .generation_owner()
                        .map(|owner| owner.stable_fingerprint()),
                    None,
                ),
            )
            .unwrap(),
            source,
        )],
        untouched: 4,
        unrecoverable: Vec::new(),
        basis_identity: [0x82; 32],
        authority_identity: world.authority.authority_identity(),
        security_scope: OperationalSecurityScope::from_admission(
            backup_custody(&world.authority).receipt(),
        ),
    };
    let target = world.restore_directory.path().join("isolated-repair");
    std::fs::create_dir_all(&target).unwrap();
    let lowered = candidates
        .select_authority_affecting_staging(world.admissible, &target, u64::MAX, 4096)
        .unwrap()
        .lower_owners()
        .unwrap();
    let backend_node = lowered
        .explanation()
        .nodes()
        .iter()
        .find(|node| node.owner() == StoreOwnerKind::PhysicalBackend)
        .unwrap()
        .identity();
    let ready = lowered
        .authorize(
            &ExactAuthorizationPort {
                substitute_plan: None,
            },
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &world.control,
            OperationalTransitionId::new("repair-isolated-crash-ready").unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap();
    assert!(matches!(
        ready.execute_with_control(
            &crate::phase_7_13_tests::CurrentStagingAuthorizationPort,
            &InterruptAt::once(
                backend_node,
                RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
            ),
        ),
        Err(super::AuthorityAffectingRepairExecutionDenial::Interrupted(
            _
        ))
    ));
    let handle = repair_handle(&world.authority, &world.control);
    assert_eq!(
        handle.recovery_disposition(),
        RepairRecoveryDisposition::NonCurrentResidueRemainsIsolated {
            durable_owner_effects: 1,
        }
    );
    let receipt = handle
        .retain_isolated_non_current_residue(&world.control, &world.authority, [0x83; 32])
        .expect("an explicit policy retains already isolated non-current residue");
    assert_eq!(receipt.basis(), [0x83; 32]);
    assert!(repair_handles(&world.authority, &world.control).is_empty());
}

fn lowered_repair(
    authority: &worth_store_authority::StoreCurrentAuthorityWitness,
    target: &std::path::Path,
    replacement: &std::path::Path,
) -> crate::LoweredRepairOwnerPlanDag {
    let damaged_digest = digest(b"damaged layout");
    let candidates = RepairCandidateSet {
        operation_id: OperationalOperationId::new("repair-effect-crash").unwrap(),
        damaged: vec![super::resolved_region::ResolvedRepairRegion::new(
            IntegrityRepairRegion::bounded(
                [0x61; 32],
                0,
                14,
                IntegrityRepairRegionClass::DerivedRebuildable,
                damaged_digest,
                physical_target_identity(target).unwrap(),
                IntegrityRepairOwnerBinding::observed(
                    IntegrityRepairArtifactFamily::LayoutIndex,
                    Some(7),
                    None,
                    None,
                ),
            )
            .unwrap(),
            target.to_path_buf(),
        )],
        untouched: 0,
        unrecoverable: Vec::new(),
        basis_identity: [0x62; 32],
        authority_identity: authority.authority_identity(),
        security_scope: OperationalSecurityScope::from_admission(
            backup_custody(authority).receipt(),
        ),
    };
    candidates
        .select_derived_maintenance(vec![DerivedIndexRepairRequest::new(
            [0x63; 32],
            target,
            damaged_digest,
            replacement,
            digest(b"rebuilt layout"),
            7,
            8,
            4096,
        )])
        .unwrap()
        .lower_owners()
        .unwrap()
}

fn repair_handle(
    authority: &worth_store_authority::StoreCurrentAuthorityWitness,
    control: &crate::OperationalControlStore,
) -> crate::IndeterminateRepairRecoveryHandle {
    repair_handles(authority, control)
        .into_iter()
        .next()
        .unwrap()
}

fn repair_handles(
    authority: &worth_store_authority::StoreCurrentAuthorityWitness,
    control: &crate::OperationalControlStore,
) -> Vec<crate::IndeterminateRepairRecoveryHandle> {
    let selection = ExactControlSelection::current(authority, control);
    let fencing = worth_store_authority::ControlStoreFencingAuthority::for_current_store(
        authority, &selection,
    );
    let crate::ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing)
    else {
        panic!("selected repair control history");
    };
    selected.indeterminate_repair_recovery_handles().to_vec()
}

struct InterruptAt {
    node: OwnerPlanNodeIdentity,
    moment: RepairExecutionBoundaryMoment,
    fired: Cell<bool>,
}

impl InterruptAt {
    fn once(node: OwnerPlanNodeIdentity, moment: RepairExecutionBoundaryMoment) -> Self {
        Self {
            node,
            moment,
            fired: Cell::new(false),
        }
    }
}

impl RepairExecutionControlPort for InterruptAt {
    fn observe(&self, boundary: RepairExecutionBoundary) -> Result<(), RepairExecutionInterrupted> {
        if !self.fired.get() && boundary.node() == self.node && boundary.moment() == self.moment {
            self.fired.set(true);
            Err(RepairExecutionInterrupted::at(boundary))
        } else {
            Ok(())
        }
    }
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
