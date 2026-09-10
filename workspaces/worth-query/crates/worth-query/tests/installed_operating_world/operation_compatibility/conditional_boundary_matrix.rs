use worth_query::facade::domain;

use super::{bind, node};
use crate::suite::conditional_node_contract::dependency;
use crate::suite::installed_operation_fixture::conditional_workspace::{
    conditional_controlled_workspace, ConditionalDonorWorkspaceScenario,
    ConditionalWorkspacePlacement,
};
use crate::suite::installed_operation_fixture::{
    conditional_controlled_workspace_with_donor, conditional_workspace, DirectConditionalCompute,
    GeometryDomain, ReadFamily, ReadVertex,
};

struct AlternateRefresh;
impl domain::WorthQueryOnDemandTriggerFamily for AlternateRefresh {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.alternate-refresh";
}

struct ManualRefresh;
impl domain::WorthQueryOnDemandTriggerFamily for ManualRefresh {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.manual-refresh";
}

#[test]
fn portable_condition_trigger_and_temporal_drift_keep_exact_owner_dimensions() {
    let source = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let cases = [
        (
            seam_node(
                domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([source.clone()])
                    .unwrap(),
                domain::WorthQueryConditionalTrigger::DependencyChange,
                domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            ),
            seam_node(
                domain::WorthQueryConditionalEvaluationCondition::always_eligible(),
                domain::WorthQueryConditionalTrigger::DependencyChange,
                domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            ),
            domain::WorthQueryPortableConditionalDimension::ConditionClass,
        ),
        (
            seam_node(
                domain::WorthQueryConditionalEvaluationCondition::on_demand(),
                domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
                domain::WorthQueryMaintenancePosture::OnDemandOnly,
            ),
            seam_node(
                domain::WorthQueryConditionalEvaluationCondition::on_demand(),
                domain::WorthQueryConditionalTrigger::on_demand::<AlternateRefresh>(),
                domain::WorthQueryMaintenancePosture::OnDemandOnly,
            ),
            domain::WorthQueryPortableConditionalDimension::Trigger,
        ),
        (
            seam_node(
                domain::WorthQueryConditionalEvaluationCondition::temporal(
                    domain::WorthQueryTemporalCondition::IntervalNanoseconds(1_000),
                ),
                domain::WorthQueryConditionalTrigger::Temporal(
                    domain::WorthQueryTemporalWake::MonotonicClock,
                ),
                domain::WorthQueryMaintenancePosture::Temporal,
            ),
            seam_node(
                domain::WorthQueryConditionalEvaluationCondition::temporal(
                    domain::WorthQueryTemporalCondition::IntervalNanoseconds(2_000),
                ),
                domain::WorthQueryConditionalTrigger::Temporal(
                    domain::WorthQueryTemporalWake::MonotonicClock,
                ),
                domain::WorthQueryMaintenancePosture::Temporal,
            ),
            domain::WorthQueryPortableConditionalDimension::TemporalCondition,
        ),
    ];

    for (index, (subject, candidate, expected)) in cases.into_iter().enumerate() {
        let left =
            conditional_workspace(&format!("conditional-portable-left-{index}"), subject).unwrap();
        let right =
            conditional_workspace(&format!("conditional-portable-right-{index}"), candidate)
                .unwrap();
        let left = bind(&left, &left.domain(GeometryDomain).unwrap());
        let right = bind(&right, &right.domain(GeometryDomain).unwrap());

        let denial = left.compatible_basis_with(&right).unwrap_err();
        assert_eq!(
            denial.kind(),
            domain::WorthQueryCompatibilityDenialKind::PortableConditionalMismatched
        );
        let Some(domain::WorthQueryOperationConditionalDimension::Declaration {
            dimension, ..
        }) = denial.portable_conditional_dimension()
        else {
            panic!("portable conditional drift must retain its owner dimension");
        };
        assert_eq!(dimension, &expected);
        assert!(denial.canonical_mismatch().is_some());
        assert_eq!(denial.counters().conditional_lowerings_compared, 0);
        assert_eq!(denial.counters().lower_runtime_contacts, 0);
    }
}

#[test]
fn replaced_lowering_inventory_keeps_query_owned_conditional_mismatch_evidence() {
    let source = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let owner_node = seam_node(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([source]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    );
    let donor_node = seam_node(
        domain::WorthQueryConditionalEvaluationCondition::always_eligible(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    );
    let mut owner =
        conditional_controlled_workspace("installed-conditional-owner", owner_node).unwrap();
    let donor = conditional_workspace("installed-conditional-donor", donor_node).unwrap();
    let installed = owner.domain(GeometryDomain).unwrap();
    let subject = bind(&owner, &installed);

    owner
        .replace_conditional_lowerings_from::<GeometryDomain, ReadVertex, ReadFamily>(&donor)
        .unwrap();
    let candidate = bind(&owner, &installed);
    let denial = subject.compatible_basis_with(&candidate).unwrap_err();

    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::PortableConditionalMismatched
    );
    assert_eq!(
        denial.installed_conditional_dimension(),
        Some(&domain::WorthQueryPortableConditionalDimension::ConditionClass)
    );
    assert!(denial.canonical_mismatch().is_some());
    assert!(denial.counters().conditional_foundational_comparisons > 0);
    assert_eq!(denial.counters().lower_runtime_contacts, 0);
}

#[test]
fn relationship_oracles_select_affinity_or_continuity_at_the_bridge_boundary() {
    let declaration = node(
        "conditional-owner-selector",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let (mut owner, donor) =
        conditional_controlled_workspace_with_donor(ConditionalDonorWorkspaceScenario {
            owner: ConditionalWorkspacePlacement {
                name: "conditional-owner-selector-owner",
                partition: "geometry-signal",
            },
            donor: ConditionalWorkspacePlacement {
                name: "conditional-owner-selector-donor",
                partition: "geometry-signal",
            },
            node: declaration,
            donor_compute: DirectConditionalCompute,
        })
        .unwrap();
    let installed = owner.domain(GeometryDomain).unwrap();
    let subject = bind(&owner, &installed);

    owner
        .replace_conditional_lowerings_from::<GeometryDomain, ReadVertex, ReadFamily>(&donor)
        .unwrap();
    let candidate = bind(&owner, &installed);

    subject.compatible_basis_with(&candidate).unwrap();
    assert_query_correspondence_affinity(subject.same_installation_with(&candidate).unwrap_err());
    assert_query_correspondence_affinity(subject.replacement_with(&candidate).unwrap_err());
    assert_query_correspondence_affinity(subject.execution_sharing_with(&candidate).unwrap_err());
}

#[test]
fn provider_semantic_drift_is_preserved_through_continuity_and_affinity_denials() {
    let declaration = node(
        "conditional-provider-seam",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let (mut owner, donor) =
        conditional_controlled_workspace_with_donor(ConditionalDonorWorkspaceScenario {
            owner: ConditionalWorkspacePlacement {
                name: "conditional-provider-owner",
                partition: "geometry-signal",
            },
            donor: ConditionalWorkspacePlacement {
                name: "conditional-provider-donor",
                partition: "geometry-signal",
            },
            node: declaration,
            donor_compute: SemanticallyDifferentCompute,
        })
        .unwrap();
    let installed = owner.domain(GeometryDomain).unwrap();
    let subject = bind(&owner, &installed);

    owner
        .replace_conditional_lowerings_from::<GeometryDomain, ReadVertex, ReadFamily>(&donor)
        .unwrap();
    let candidate = bind(&owner, &installed);

    let continuity = subject.compatible_basis_with(&candidate).unwrap_err();
    assert!(matches!(
        continuity.conditional_continuity_mismatch(),
        Some(
            worth_runtime_bridge::facade::BridgeConditionalContinuityMismatch::ProviderSemanticContract {
                role: worth_runtime_bridge::facade::BridgeConditionalProviderRole::Compute
            }
        )
    ), "{:?}", continuity.conditional_continuity_mismatch());
    assert_eq!(continuity.counters().lower_runtime_contacts, 0);

    let affinity = subject.replacement_with(&candidate).unwrap_err();
    assert!(matches!(
        affinity.conditional_affinity_mismatch(),
        Some(
            worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch::Continuity(
                worth_runtime_bridge::facade::BridgeConditionalContinuityMismatch::ProviderSemanticContract {
                    role: worth_runtime_bridge::facade::BridgeConditionalProviderRole::Compute
                }
            )
        )
    ));
    assert_eq!(affinity.counters().lower_runtime_contacts, 0);
}

fn seam_node(
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

fn assert_query_correspondence_affinity<T>(denial: T)
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

trait AffinityDenial {
    fn kind(&self) -> domain::WorthQueryCompatibilityDenialKind;
    fn conditional_affinity_mismatch(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeConditionalExecutionAffinityMismatch>;
    fn lower_runtime_contacts(&self) -> usize;
}

macro_rules! affinity_denial {
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

affinity_denial!(domain::WorthQuerySameInstallationDenial);
affinity_denial!(domain::WorthQueryReplacementDenial);
affinity_denial!(domain::WorthQueryExecutionSharingDenial);

struct SemanticallyDifferentCompute;

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
        crate::suite::installed_operation_fixture::execution_resource_support()
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
