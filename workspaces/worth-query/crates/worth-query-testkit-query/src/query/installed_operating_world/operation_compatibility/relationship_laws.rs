pub use worth_query::facade::domain;

pub use super::hostile_matrix::fixture::{bind_branch, bind_current, no_primary_read_runtime};
pub use super::{bind, node};
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::conditional_workspace::{
    ConditionalDonorWorkspaceScenario, ConditionalWorkspacePlacement,
};
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{
    conditional_controlled_workspace_with_donor, required_domain_runtime, AuxiliaryDomain,
    DirectConditionalCompute, GeometryDomain, ReadFamily, ReadVertex,
};

pub fn assert_continuity_drift<T>(denial: T)
where
    T: ConditionalContinuityDriftDenial,
{
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::ConditionalLowering
    );
    assert!(matches!(
        denial.conditional_continuity_mismatch(),
        Some(
            worth_runtime_bridge::facade::BridgeConditionalContinuityMismatch::TargetMeaning {
                ordinal: 0,
                target: 0,
            }
        )
    ));
    assert_eq!(denial.lower_runtime_contacts(), 0);
}

pub fn assert_affinity_drift<T>(denial: T)
where
    T: ConditionalAffinityDriftDenial,
{
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::ConditionalLowering
    );
    assert!(matches!(
        denial.conditional_affinity_mismatch(),
        Some(
            worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch::Continuity(
                worth_runtime_bridge::facade::BridgeConditionalContinuityMismatch::TargetMeaning {
                    ordinal: 0,
                    target: 0,
                }
            )
        )
    ));
    assert_eq!(denial.lower_runtime_contacts(), 0);
}

pub trait ConditionalContinuityDriftDenial {
    fn kind(&self) -> domain::WorthQueryCompatibilityDenialKind;
    fn conditional_continuity_mismatch(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalContinuityMismatch>;
    fn lower_runtime_contacts(&self) -> usize;
}

#[macro_export]
macro_rules! worlds_query_installed_operating_world_operation_compatibility_relationship_laws_rs_conditional_continuity_drift_denial {
    ($type:ty) => {
        impl ConditionalContinuityDriftDenial for $type {
            fn kind(&self) -> domain::WorthQueryCompatibilityDenialKind {
                self.kind()
            }
            fn conditional_continuity_mismatch(
                &self,
            ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalContinuityMismatch> {
                self.conditional_continuity_mismatch()
            }
            fn lower_runtime_contacts(&self) -> usize {
                self.counters().lower_runtime_contacts
            }
        }
    };
}
pub use crate::worlds_query_installed_operating_world_operation_compatibility_relationship_laws_rs_conditional_continuity_drift_denial as conditional_continuity_drift_denial;

conditional_continuity_drift_denial!(domain::WorthQueryBasisCompatibilityDenial);
conditional_continuity_drift_denial!(domain::WorthQueryRebindCompatibilityDenial);

pub trait ConditionalAffinityDriftDenial {
    fn kind(&self) -> domain::WorthQueryCompatibilityDenialKind;
    fn conditional_affinity_mismatch(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch>;
    fn lower_runtime_contacts(&self) -> usize;
}

#[macro_export]
macro_rules! worlds_query_installed_operating_world_operation_compatibility_relationship_laws_rs_conditional_affinity_drift_denial {
    ($type:ty) => {
        impl ConditionalAffinityDriftDenial for $type {
            fn kind(&self) -> domain::WorthQueryCompatibilityDenialKind {
                self.kind()
            }
            fn conditional_affinity_mismatch(
                &self,
            ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch>
            {
                self.conditional_affinity_mismatch()
            }
            fn lower_runtime_contacts(&self) -> usize {
                self.counters().lower_runtime_contacts
            }
        }
    };
}
pub use crate::worlds_query_installed_operating_world_operation_compatibility_relationship_laws_rs_conditional_affinity_drift_denial as conditional_affinity_drift_denial;

conditional_affinity_drift_denial!(domain::WorthQuerySameInstallationDenial);
conditional_affinity_drift_denial!(domain::WorthQueryReplacementDenial);
conditional_affinity_drift_denial!(domain::WorthQueryExecutionSharingDenial);
