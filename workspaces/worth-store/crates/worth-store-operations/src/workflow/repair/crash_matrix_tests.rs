use std::cell::Cell;

use super::integrity_classification::{
    IntegrityRepairArtifactFamily, IntegrityRepairOwnerBinding, IntegrityRepairRegion,
    IntegrityRepairRegionClass,
};

use super::intent::physical_target_identity;
use super::resolved_region::ResolvedRepairRegion;
use super::{
    AuthorityAffectingRepairExecutionDenial, RepairCandidateSet, RepairExecutionBoundary,
    RepairExecutionBoundaryMoment, RepairExecutionControlPort, RepairExecutionInterrupted,
};
use crate::phase_1_6_tests::support::backup_custody;
use crate::phase_7_13_tests::{
    operator_assertion, CurrentStagingAuthorizationPort, ExactAuthorizationPort,
    ExactControlSelection,
};
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalSecurityScope, OperationalTransitionId, OwnerPlanNodeIdentity,
};

#[test]
fn every_authority_repair_owner_boundary_recovers_to_exact_completion() {
    let probe = lowered_world("repair-crash-matrix-probe");
    let boundaries = probe
        .lowered
        .explanation()
        .nodes()
        .iter()
        .flat_map(|node| {
            [
                RepairExecutionBoundaryMoment::BeforeOwnerEffect,
                RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
                RepairExecutionBoundaryMoment::AfterReceiptPersistence,
            ]
            .map(|moment| (node.owner(), node.effect(), moment))
        })
        .collect::<Vec<_>>();
    assert!(
        boundaries.len() >= 12,
        "authority repair must retain its multi-owner DAG"
    );
    drop(probe);

    for (index, (owner, effect, moment)) in boundaries.into_iter().enumerate() {
        let case = format!("repair-crash-matrix-{index}");
        let MatrixWorld {
            lowered,
            authority,
            control,
            _scenario,
            _restore_directory,
        } = lowered_world(&case);
        let node = lowered
            .explanation()
            .nodes()
            .iter()
            .find(|node| node.owner() == owner && node.effect() == effect)
            .expect("the rebuilt canonical DAG retains the semantic owner node")
            .identity();
        let restart = lowered.clone();
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
                OperationalTransitionId::new(format!("{case}/ready")).unwrap(),
                &authority,
                21,
                AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
            )
            .unwrap();
        let interrupted = ready.execute_with_control(
            &CurrentStagingAuthorizationPort,
            &InterruptAt::once(node, moment),
        );
        assert!(
            matches!(
                &interrupted,
                Err(AuthorityAffectingRepairExecutionDenial::Interrupted(_))
            ),
            "boundary {index} {node:?} {moment:?} returned {interrupted:?}"
        );
        let handle = repair_handle(&authority, &control);
        restart
            .recover_ready(&handle, &control, &authority)
            .expect("exact plan rebinds to the durable owner journal")
            .execute(&CurrentStagingAuthorizationPort)
            .expect("every owner cut converges after replay");
        assert!(repair_handles(&authority, &control).is_empty());
    }
}

struct MatrixWorld {
    lowered: super::LoweredAuthorityAffectingRepairOwnerPlanDag,
    authority: worth_store_authority::StoreCurrentAuthorityWitness,
    control: crate::OperationalControlStore,
    _scenario: crate::phase_1_6_tests::support::BackupScenario,
    _restore_directory: tempfile::TempDir,
}

fn lowered_world(case: &str) -> MatrixWorld {
    let world = crate::phase_7_13_tests::restore_world(case);
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
        operation_id: OperationalOperationId::new(case).unwrap(),
        damaged: vec![ResolvedRepairRegion::new(
            IntegrityRepairRegion::bounded(
                [0x91; 32],
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
        basis_identity: [0x92; 32],
        authority_identity: world.authority.authority_identity(),
        security_scope: OperationalSecurityScope::from_admission(
            backup_custody(&world.authority).receipt(),
        ),
    };
    let target = world.restore_directory.path().join("repair-matrix-target");
    std::fs::create_dir_all(&target).unwrap();
    let lowered = candidates
        .select_authority_affecting_staging(world.admissible, target, u64::MAX, 4096)
        .unwrap()
        .lower_owners()
        .unwrap();
    MatrixWorld {
        lowered,
        authority: world.authority,
        control: world.control,
        _scenario: world.scenario,
        _restore_directory: world.restore_directory,
    }
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
