use crate::data::comparator::{InstalledSignalComparatorRole, VersionComparatorPolicy};
use crate::data::node::{EvaluationCondition, InstalledSignalConditionRole};

use super::dependency_versions::SignalConditionalDependencyVersion;
use super::{
    InstalledSignalConditionalContract, SignalConditionalArtifactReusePolicy,
    SignalConditionalCondition, SignalConditionalDecisionClass, SignalThresholdBoundary,
    SignalThresholdComparisonDomain, SignalThresholdValueFamily,
};

mod allocation;
mod authority;

pub(super) use authority::{
    mint_signal_conditional_decision_identity, SignalConditionalDecisionAuthorityIdentity,
};
pub use authority::{
    SignalConditionalDecisionIdentityKind, SignalConditionalDecisionProjectionIdentity,
};

#[cfg(test)]
mod tests;

pub(super) fn decision_projection_basis(
    contract: &InstalledSignalConditionalContract,
    snapshot_identity: &str,
    execution_identity: &str,
    attempt: u64,
    class: SignalConditionalDecisionClass,
    dependencies: &[SignalConditionalDependencyVersion],
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<String, crate::data::error::SignalError> {
    let capacity = allocation::admit(
        contract,
        snapshot_identity,
        execution_identity,
        dependencies,
        work,
    )?;
    let node = contract.node();
    let mut material = String::with_capacity(capacity);
    material.push_str("signal-conditional-decision-v1");
    field(&mut material, "graph", contract.graph_instance_id());
    field(&mut material, "node-index", node.index());
    field(&mut material, "node-generation", node.generation());
    text_field(&mut material, "snapshot", snapshot_identity);
    text_field(&mut material, "execution", execution_identity);
    field(&mut material, "attempt", attempt);
    token(&mut material, "decision-class", decision_class(class));
    field(&mut material, "dependency-width", dependencies.len());
    for dependency in dependencies {
        append_dependency(&mut material, dependency);
    }
    material.push_str(contract.projection_contract());
    debug_assert!(material.len() <= capacity);
    Ok(material)
}

pub(super) fn contract_projection_basis(contract: &InstalledSignalConditionalContract) -> String {
    let mut material = String::new();
    append_contract(&mut material, contract);
    material
}

fn append_dependency(material: &mut String, dependency: &SignalConditionalDependencyVersion) {
    field(material, "dependency-node-index", dependency.node.index());
    field(
        material,
        "dependency-node-generation",
        dependency.node.generation(),
    );
    field(material, "dependency-aspect", dependency.aspect.index());
    match dependency.scope.as_ref() {
        None => token(material, "dependency-scope", "none"),
        Some(scope) => {
            token(material, "dependency-scope", "partition");
            text_field(material, "dependency-partition", scope.partition.0.as_str());
            match scope.detail.as_deref() {
                Some(detail) => text_field(material, "dependency-detail", detail),
                None => token(material, "dependency-detail", "none"),
            }
            token(
                material,
                "dependency-match",
                match scope.match_mode {
                    crate::data::output::PartitionMatchMode::WholePartition => "whole",
                    crate::data::output::PartitionMatchMode::PartitionAndDetail => "detail",
                },
            );
        }
    }
    field(material, "dependency-version", dependency.version);
}

fn append_contract(material: &mut String, contract: &InstalledSignalConditionalContract) {
    field(material, "contract-generation", contract.generation());
    field(material, "contract-occurrence", contract.occurrence());
    append_installed_condition(material, contract.condition());
    append_semantic_condition(material, contract.semantic_condition());
    field(
        material,
        "dependency-aspects",
        contract.dependency_aspects().bits(),
    );
    field(
        material,
        "trigger-aspects",
        contract.trigger_aspects().bits(),
    );
    append_comparator(
        material,
        "dependency-comparator",
        contract.dependency_comparator(),
    );
    append_comparator(material, "output-comparator", contract.output_comparator());
    append_reuse(material, contract.artifact_reuse());
}

fn append_installed_condition(material: &mut String, condition: &EvaluationCondition) {
    match condition {
        EvaluationCondition::Always => token(material, "installed-condition", "always"),
        EvaluationCondition::AspectFilter(mask) => {
            token(material, "installed-condition", "aspect-filter");
            field(material, "installed-condition-aspects", mask.bits());
        }
        EvaluationCondition::OnDemand => token(material, "installed-condition", "on-demand"),
        EvaluationCondition::Installed(identity) => {
            token(material, "installed-condition", "owner-installed");
            field(material, "condition-graph", identity.graph_instance_id());
            token(
                material,
                "condition-role",
                match identity.role() {
                    InstalledSignalConditionRole::Predicate => "predicate",
                    InstalledSignalConditionRole::TemporalWake => "temporal-wake",
                },
            );
        }
        EvaluationCondition::DeltaThreshold(_)
        | EvaluationCondition::Temporal(_)
        | EvaluationCondition::Custom(_) => unreachable!(
            "installed conditional contracts are created only by Signal's typed lowering"
        ),
    }
}

fn append_semantic_condition(material: &mut String, condition: &SignalConditionalCondition) {
    match condition {
        SignalConditionalCondition::Always => token(material, "semantic-condition", "always"),
        SignalConditionalCondition::AspectFilter(mask) => {
            token(material, "semantic-condition", "aspect-filter");
            field(material, "semantic-condition-aspects", mask.bits());
        }
        SignalConditionalCondition::DeltaThreshold(threshold) => {
            token(material, "semantic-condition", "delta-threshold");
            let basis = worth_foundational::facade::prepare_aspect_value_identity_basis(
                threshold.threshold(),
            );
            text_field(material, "threshold-value", basis.as_str());
            text_field(material, "threshold-unit", threshold.unit_identity());
            token(
                material,
                "threshold-family",
                match threshold.value_family() {
                    SignalThresholdValueFamily::Integer => "integer",
                    SignalThresholdValueFamily::Float32 => "float32",
                    SignalThresholdValueFamily::Float64 => "float64",
                },
            );
            token(
                material,
                "threshold-domain",
                match threshold.comparison_domain() {
                    SignalThresholdComparisonDomain::AbsoluteDifference => "absolute-difference",
                    SignalThresholdComparisonDomain::RelativeRatio => "relative-ratio",
                },
            );
            token(
                material,
                "threshold-boundary",
                match threshold.boundary() {
                    SignalThresholdBoundary::Inclusive => "inclusive",
                    SignalThresholdBoundary::Exclusive => "exclusive",
                },
            );
        }
        SignalConditionalCondition::OnDemand => token(material, "semantic-condition", "on-demand"),
        SignalConditionalCondition::RuntimePredicate => {
            token(material, "semantic-condition", "runtime-predicate")
        }
        SignalConditionalCondition::TemporalWake => {
            token(material, "semantic-condition", "temporal-wake")
        }
    }
}

fn append_comparator(material: &mut String, label: &'static str, policy: &VersionComparatorPolicy) {
    match policy {
        VersionComparatorPolicy::Exact => token(material, label, "exact"),
        VersionComparatorPolicy::Tolerance { epsilon } => {
            token(material, label, "tolerance");
            field(material, "comparator-epsilon", *epsilon);
        }
        VersionComparatorPolicy::OutputIdentity => token(material, label, "output-identity"),
        VersionComparatorPolicy::Custom { key } => {
            token(material, label, "custom");
            text_field(material, "comparator-key", key);
        }
        VersionComparatorPolicy::Installed { identity } => {
            token(material, label, "owner-installed");
            field(material, "comparator-graph", identity.graph_instance_id());
            token(
                material,
                "comparator-role",
                comparator_role(identity.role()),
            );
        }
    }
}

fn append_reuse(material: &mut String, reuse: &SignalConditionalArtifactReusePolicy) {
    match reuse {
        SignalConditionalArtifactReusePolicy::NotReusable => {
            token(material, "artifact-reuse", "not-reusable")
        }
        SignalConditionalArtifactReusePolicy::DependencyAndOutputEquivalent => token(
            material,
            "artifact-reuse",
            "dependency-and-output-equivalent",
        ),
        SignalConditionalArtifactReusePolicy::OutputEquivalent => {
            token(material, "artifact-reuse", "output-equivalent")
        }
        SignalConditionalArtifactReusePolicy::Installed(identity) => {
            token(material, "artifact-reuse", "owner-installed");
            field(material, "reuse-graph", identity.graph_instance_id());
            token(material, "reuse-role", comparator_role(identity.role()));
        }
    }
}

const fn comparator_role(role: InstalledSignalComparatorRole) -> &'static str {
    match role {
        InstalledSignalComparatorRole::DependencyVersion => "dependency-version",
        InstalledSignalComparatorRole::OutputEquivalence => "output-equivalence",
        InstalledSignalComparatorRole::ArtifactReuse => "artifact-reuse",
    }
}

const fn decision_class(class: SignalConditionalDecisionClass) -> &'static str {
    match class {
        SignalConditionalDecisionClass::ComputedChanged => "computed-changed",
        SignalConditionalDecisionClass::ComputedRevertedClean => "computed-reverted-clean",
        SignalConditionalDecisionClass::DependencyUnchanged => "dependency-unchanged",
        SignalConditionalDecisionClass::SuppressedBeforeCompute => "suppressed-before-compute",
        SignalConditionalDecisionClass::DeferredByCondition => "deferred-by-condition",
        SignalConditionalDecisionClass::DeferredTemporal => "deferred-temporal",
        SignalConditionalDecisionClass::DeferredOnDemand => "deferred-on-demand",
    }
}

fn token(material: &mut String, label: &'static str, value: &'static str) {
    text_field(material, label, value);
}

fn field(material: &mut String, label: &'static str, value: impl std::fmt::Display) {
    text_field(material, label, &value.to_string());
}

fn text_field(material: &mut String, label: &'static str, value: &str) {
    use std::fmt::Write;
    write!(
        material,
        "|{}:{label}={}:{}",
        label.len(),
        value.len(),
        value
    )
    .expect("writing canonical conditional identity material cannot fail");
}
