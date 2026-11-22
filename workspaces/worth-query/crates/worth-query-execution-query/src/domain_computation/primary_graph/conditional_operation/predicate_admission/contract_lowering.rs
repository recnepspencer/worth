//! Lower declared condition, equivalence and location meaning into Bridge contracts.
use super::bridge_denial;
use super::WorthQueryConditionalRuntimeInstallationDenial;
use std::sync::Arc;
use worth_query_installation::facade::{
    WorthQueryArtifactReuseEquivalence, WorthQueryComparatorRequirement,
    WorthQueryConditionalConditionClass, WorthQueryOutputEquivalenceRequirement,
    WorthQueryPortableConditionalNodeDeclaration,
};
use worth_runtime_bridge::facade::{
    BridgeConditionalCondition, BridgeConditionalContract, BridgeConditionalContractParts,
    BridgeConditionalLocation,
};
use worth_signal::facade::{SignalConditionalArtifactReuse, SignalConditionalVersionComparator};
pub(super) fn lower_temporal_contract(
    declaration: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<BridgeConditionalContract, WorthQueryConditionalRuntimeInstallationDenial> {
    if declaration.condition().class() != WorthQueryConditionalConditionClass::Temporal {
        return Err(bridge_denial(
            "named clock binding did not retain a temporal condition",
        ));
    }
    Ok(BridgeConditionalContract::new(
        BridgeConditionalContractParts {
            identity: Arc::from(declaration.identity()),
            dependency_count: declaration.dependencies().len(),
            condition_dependency_ordinals: (0..declaration.dependencies().len()).collect(),
            condition: BridgeConditionalCondition::TemporalWake,
            dependency_comparator: lower_dependency_comparator(declaration.dependency_comparator()),
            output_comparator: lower_output_comparator(declaration.output_equivalence()),
            artifact_reuse: lower_artifact_reuse(declaration.artifact_reuse_equivalence()),
        },
    ))
}

pub(super) fn lower_dependency_contract(
    declaration: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<BridgeConditionalContract, WorthQueryConditionalRuntimeInstallationDenial> {
    if declaration.dependencies().is_empty() {
        return Err(bridge_denial(
            "output readiness conditional has no declared dependency",
        ));
    }
    let condition_dependency_ordinals = declaration
        .condition()
        .dependencies()
        .iter()
        .map(|condition_dependency| {
            declaration
                .dependencies()
                .iter()
                .position(|dependency| dependency == condition_dependency)
                .ok_or_else(|| bridge_denial("readiness condition dependency was not declared"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if condition_dependency_ordinals.is_empty() {
        return Err(bridge_denial(
            "output readiness condition has no declared observation dependency",
        ));
    }
    Ok(BridgeConditionalContract::new(
        BridgeConditionalContractParts {
            identity: Arc::from(declaration.identity()),
            dependency_count: declaration.dependencies().len(),
            condition_dependency_ordinals,
            condition: BridgeConditionalCondition::RuntimePredicate,
            dependency_comparator: lower_dependency_comparator(declaration.dependency_comparator()),
            output_comparator: lower_output_comparator(declaration.output_equivalence()),
            artifact_reuse: lower_artifact_reuse(declaration.artifact_reuse_equivalence()),
        },
    ))
}

pub(super) fn lower_location(
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
) -> BridgeConditionalLocation {
    match location.stage_identity() {
        Some(stage) => BridgeConditionalLocation::workflow_stage(
            Arc::from(stage),
            Arc::from(location.node_identity()),
        ),
        None => BridgeConditionalLocation::operation(Arc::from(location.node_identity())),
    }
}

fn lower_dependency_comparator(
    requirement: &WorthQueryComparatorRequirement,
) -> SignalConditionalVersionComparator {
    match requirement {
        WorthQueryComparatorRequirement::ExactCanonicalValue
        | WorthQueryComparatorRequirement::FoundationalContractEquivalence => {
            SignalConditionalVersionComparator::Exact
        }
        WorthQueryComparatorRequirement::Registered(_) => {
            SignalConditionalVersionComparator::RuntimeResolved
        }
    }
}

fn lower_output_comparator(
    requirement: &WorthQueryOutputEquivalenceRequirement,
) -> SignalConditionalVersionComparator {
    match requirement {
        WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue
        | WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence => {
            SignalConditionalVersionComparator::Exact
        }
        WorthQueryOutputEquivalenceRequirement::OutputIdentity => {
            SignalConditionalVersionComparator::OutputIdentity
        }
        WorthQueryOutputEquivalenceRequirement::Registered(_) => {
            SignalConditionalVersionComparator::RuntimeResolved
        }
    }
}

fn lower_artifact_reuse(
    requirement: &WorthQueryArtifactReuseEquivalence,
) -> SignalConditionalArtifactReuse {
    match requirement {
        WorthQueryArtifactReuseEquivalence::NotReusable => {
            SignalConditionalArtifactReuse::NotReusable
        }
        WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent => {
            SignalConditionalArtifactReuse::DependencyAndOutputEquivalent
        }
        WorthQueryArtifactReuseEquivalence::OutputEquivalent => {
            SignalConditionalArtifactReuse::OutputEquivalent
        }
        WorthQueryArtifactReuseEquivalence::Registered(_) => {
            SignalConditionalArtifactReuse::RuntimeResolved
        }
    }
}
