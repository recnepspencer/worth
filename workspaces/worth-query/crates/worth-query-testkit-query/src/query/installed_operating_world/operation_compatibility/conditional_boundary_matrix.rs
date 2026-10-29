pub use worth_query::facade::domain;

pub use super::{bind, node};
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::conditional_node_contract::dependency;
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::conditional_workspace::{
    conditional_controlled_workspace, ConditionalDonorWorkspaceScenario,
    ConditionalWorkspacePlacement,
};
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{
    conditional_controlled_workspace_with_donor, conditional_workspace, DirectConditionalCompute,
    GeometryDomain, ReadFamily, ReadVertex,
};

pub struct AlternateRefresh;
impl domain::WorthQueryOnDemandTriggerFamily for AlternateRefresh {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.alternate-refresh";
}

pub struct ManualRefresh;
impl domain::WorthQueryOnDemandTriggerFamily for ManualRefresh {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.manual-refresh";
}

pub fn seam_node(
    condition: domain::WorthQueryConditionalEvaluationCondition,
    trigger: domain::WorthQueryConditionalTrigger,
    maintenance: domain::WorthQueryMaintenancePosture,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "conditional-portable-seam",
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
    }])
    .required_context([domain::WorthQueryConditionalNodeContext::Basis])
    .evaluation(condition, trigger)
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        maintenance,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::ContributesToOperationOutput)
    .finish()
    .unwrap()
}

pub fn assert_query_correspondence_affinity<T>(denial: T)
where
    T: AffinityDenial,
{
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::ConditionalLowering
    );
    assert!(matches!(
        denial.conditional_affinity_mismatch(),
        Some(
            worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch::SourceCorrespondenceAuthority {
                ordinal: 0
            }
        )
    ), "{:?}", denial.conditional_affinity_mismatch());
    assert_eq!(denial.lower_runtime_contacts(), 0);
}

pub trait AffinityDenial {
    fn kind(&self) -> domain::WorthQueryCompatibilityDenialKind;
    fn conditional_affinity_mismatch(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch>;
    fn lower_runtime_contacts(&self) -> usize;
}

#[macro_export]
macro_rules! worlds_query_installed_operating_world_operation_compatibility_conditional_boundary_matrix_rs_affinity_denial {
    ($type:ty) => {
        impl AffinityDenial for $type {
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
pub use crate::worlds_query_installed_operating_world_operation_compatibility_conditional_boundary_matrix_rs_affinity_denial as affinity_denial;

affinity_denial!(domain::WorthQuerySameInstallationDenial);
affinity_denial!(domain::WorthQueryReplacementDenial);
affinity_denial!(domain::WorthQueryExecutionSharingDenial);

pub struct SemanticallyDifferentCompute;

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for SemanticallyDifferentCompute
{
    type SemanticContract = u64;

    fn semantic_contract(&self) -> Self::SemanticContract {
        7
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}
