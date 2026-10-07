use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_authority::StoreCurrentAuthorityWitness;

use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation,
    ExecutedAuthorityAffectingRepair, OperationalControlStore, OperationalControlStorePort,
    OperationalOperationId, OperationalSecurityScope, OperationalTransitionId,
    ProductionRestoreAdmissibleBackupBundle, RepairReadinessDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioRepairSourceDenialReceipt {
    stale_authority_denial_identity: [u8; 32],
    cross_scope_denial_identity: [u8; 32],
    evidence_identity: [u8; 32],
}

pub fn certify_scenario_repair_source_denials(case: &str) -> ScenarioRepairSourceDenialReceipt {
    let scenario = super::OwnerBackedBackupScenario::materialize(case);
    let control = scenario.control_store();
    let source = scenario
        .execute_named(case, "repair-source-current", &control)
        .into_restore_source();
    let foreign = super::OwnerBackedBackupScenario::materialize(&format!("{case}/foreign"));
    let foreign_control = foreign.control_store();
    let foreign_source = foreign
        .execute_named(case, "repair-source-foreign", &foreign_control)
        .into_restore_source();
    let stale = crate::workflow::certification_authority_repair_candidates_from_backup_observation(
        OperationalOperationId::new(format!("{case}/stale-source")).unwrap(),
        &source,
        None,
    )
    .unwrap()
    .select_authority_affecting_staging(
        foreign_source,
        scenario.workspace_root().join("stale-source-target"),
        u64::MAX,
        31,
    )
    .unwrap_err();
    assert_eq!(
        stale,
        crate::RepairResolutionDenial::StaleTrustedSourceAuthority
    );

    let wrong_scope = [0x6d; 32];
    assert_ne!(
        wrong_scope,
        source
            .custody()
            .custody_receipt()
            .identity()
            .stable_fingerprint()
    );
    let cross_scope =
        crate::workflow::certification_authority_repair_candidates_from_backup_observation(
            OperationalOperationId::new(format!("{case}/cross-scope-source")).unwrap(),
            &source,
            Some(wrong_scope),
        )
        .unwrap()
        .select_authority_affecting_staging(
            source,
            scenario.workspace_root().join("cross-scope-target"),
            u64::MAX,
            31,
        )
        .unwrap_err();
    assert_eq!(
        cross_scope,
        crate::RepairResolutionDenial::WrongTrustedSourceSecurityScope
    );
    let stale_authority_denial_identity = denial_identity(case, b"stale-authority");
    let cross_scope_denial_identity = denial_identity(case, b"cross-scope");
    let mut digest = Sha256::new();
    digest.update(b"worth-store-s10-repair-source-denials-v1");
    digest.update(stale_authority_denial_identity);
    digest.update(cross_scope_denial_identity);
    ScenarioRepairSourceDenialReceipt {
        stale_authority_denial_identity,
        cross_scope_denial_identity,
        evidence_identity: digest.finalize().into(),
    }
}

fn denial_identity(case: &str, denial: &[u8]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(case.as_bytes());
    digest.update(denial);
    digest.finalize().into()
}

impl ScenarioRepairSourceDenialReceipt {
    pub const fn stale_authority_denial_identity(self) -> [u8; 32] {
        self.stale_authority_denial_identity
    }
    pub const fn cross_scope_denial_identity(self) -> [u8; 32] {
        self.cross_scope_denial_identity
    }
    pub const fn evidence_identity(self) -> [u8; 32] {
        self.evidence_identity
    }
}

use super::{
    certification_operator_assertion, CurrentScenarioStagingPort, ExactScenarioAuthorizationPort,
};

/// Drive a derived-index repair through Operations' pure plan, owner lowering
/// and authorization, and return the readiness denial: D15 gives derived-index
/// rebuild publication to the Store rebuild owner, so Operations refuses before
/// consuming the authorization or touching the target.
pub fn deny_scenario_derived_repair(
    operation_name: &str,
    target: &Path,
    replacement: &Path,
    security_scope: OperationalSecurityScope,
    authority: &StoreCurrentAuthorityWitness,
    control: &OperationalControlStore,
    append: &dyn OperationalControlStorePort,
) -> RepairReadinessDenial {
    let lowered = crate::workflow::certification_derived_maintenance_from_fixture_observation(
        OperationalOperationId::new(operation_name).expect("repair operation identity"),
        target,
        replacement,
        authority.authority_identity(),
        security_scope,
    )
    .expect("independently observed derived repair fixture")
    .lower_owners()
    .expect("canonical derived repair owner DAG");
    assert!(
        lowered
            .explanation()
            .nodes()
            .iter()
            .any(|node| node.owner() == crate::StoreOwnerKind::LayoutIndexes),
        "Operations still schedules the layout owner in its repair plan"
    );
    lowered
        .authorize(
            &ExactScenarioAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect("exact repair authorization")
        .ready_with_certification_control_store(
            control,
            append,
            OperationalTransitionId::new(format!("{operation_name}/consume-authorization"))
                .expect("repair authorization transition"),
            authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .err()
        .expect("Store owns derived-index rebuild publication")
}

pub fn execute_scenario_authority_affecting_repair(
    operation_name: &str,
    source: ProductionRestoreAdmissibleBackupBundle,
    target_parent: &Path,
    authority: &StoreCurrentAuthorityWitness,
    control: &OperationalControlStore,
    append: &dyn OperationalControlStorePort,
) -> ExecutedAuthorityAffectingRepair {
    std::fs::create_dir_all(target_parent).expect("authority repair staging parent");
    crate::workflow::certification_authority_repair_from_backup_observation(
        OperationalOperationId::new(operation_name).expect("authority repair operation identity"),
        source,
        target_parent,
    )
    .expect("manifest-observed authority repair candidate")
    .lower_owners()
    .expect("canonical authority repair owner DAG")
    .authorize(
        &ExactScenarioAuthorizationPort,
        &certification_operator_assertion(),
        20,
        80,
        AuthorizationReplayPolicy::SingleUse,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
    )
    .expect("exact authority repair authorization")
    .ready_with_certification_control_store(
        control,
        append,
        OperationalTransitionId::new(format!("{operation_name}/consume-staging-authorization"))
            .expect("authority repair authorization transition"),
        authority,
        21,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
    )
    .expect("durable authority repair readiness")
    .execute(&CurrentScenarioStagingPort)
    .expect("five-owner authority-affecting repair")
}

#[cfg(test)]
mod tests {
    use crate::OperationalSecurityScope;

    use super::*;
    use crate::certification_scenario::OwnerBackedBackupScenario;

    #[test]
    fn scenario_derived_repair_denies_typed_before_any_operations_effect() {
        let scenario = OwnerBackedBackupScenario::materialize("derived-repair-scenario");
        let control = scenario.control_store();
        let source = scenario
            .execute("derived-repair-scenario", &control)
            .into_restore_source();
        let security_scope =
            OperationalSecurityScope::from_admission(source.custody().custody_receipt());
        let target = scenario.workspace_root().join("damaged.index");
        let replacement = scenario.workspace_root().join("replacement.index");
        std::fs::write(&target, b"damaged-derived-index").unwrap();
        std::fs::write(&replacement, b"rebuilt-derived-index").unwrap();
        let history_before = control.observe_selection_coordinates().unwrap();
        assert!(
            history_before.is_some(),
            "the backup owner effect is durable"
        );

        let denial = deny_scenario_derived_repair(
            "derived-repair-scenario/repair",
            &target,
            &replacement,
            security_scope,
            scenario.authority(),
            &control,
            &control,
        );

        assert!(matches!(
            denial,
            RepairReadinessDenial::StoreDerivedIndexRebuildRequired
        ));
        // No Operations effect: the target, the replacement and the control
        // history (no authorization consumption, no repair journal) are unchanged.
        assert_eq!(std::fs::read(&target).unwrap(), b"damaged-derived-index");
        assert_eq!(
            std::fs::read(&replacement).unwrap(),
            b"rebuilt-derived-index"
        );
        assert_eq!(
            control.observe_selection_coordinates().unwrap(),
            history_before
        );
    }
}
