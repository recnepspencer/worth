use std::sync::Arc;

use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

use super::super::super::{
    WorthQueryDomainInstallationDenial, WorthQueryDomainInstallationDenialKind,
    WorthQueryDomainInvariantDefinition, WorthQueryDomainInvariantPredicate,
};

#[derive(Clone, Debug)]
struct InstalledRequiresOutgoingRelationsRule {
    rule_id: String,
    display_name: Arc<str>,
    semantic_version: CustomInvariantSemanticVersion,
    relevant_entity_kinds: Vec<KindId>,
    required_relation_kinds: Vec<KindId>,
    traversal_depth: u16,
    maximum_work_units: std::num::NonZeroU64,
}

#[derive(Clone, Debug)]
struct RequiresOutgoingRelationsScope {
    visible_relevant_entities: Vec<EntityId>,
    planned_relevant_entity_count: usize,
    planned_relevant_relation_count: usize,
    traversal_exhausted: bool,
}

pub(super) fn compile_invariant_definition(
    provenance: &crate::runtime::WorthQueryInstalledDomainSubstrateProvenance,
    definition: &WorthQueryDomainInvariantDefinition,
) -> Result<CustomInvariantRegistration, WorthQueryDomainInstallationDenial> {
    let semantic_version = CustomInvariantSemanticVersion::new(
        u16::try_from(definition.semantic_version().major()).map_err(|_| {
            WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed,
                format!(
                    "{} major version exceeds invariant runtime range",
                    definition.name().as_str()
                ),
            )
        })?,
        u16::try_from(definition.semantic_version().minor()).map_err(|_| {
            WorthQueryDomainInstallationDenial::new(
                WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed,
                format!(
                    "{} minor version exceeds invariant runtime range",
                    definition.name().as_str()
                ),
            )
        })?,
    );
    let rule = match definition.predicate() {
        WorthQueryDomainInvariantPredicate::RequiresOutgoingRelations {
            relevant_entity_kinds,
            required_relation_kinds,
            traversal_depth,
        } => InstalledRequiresOutgoingRelationsRule {
            rule_id: format!(
                "{}.package-{}.{}.{}",
                provenance.domain_owner(),
                provenance.package_version(),
                provenance.package_identity(),
                definition.name().as_str()
            ),
            display_name: Arc::from(format!(
                "{} package {} {}",
                provenance.domain_owner(),
                provenance.package_version(),
                definition.name().as_str()
            )),
            semantic_version,
            relevant_entity_kinds: relevant_entity_kinds.clone(),
            required_relation_kinds: required_relation_kinds.clone(),
            traversal_depth: *traversal_depth,
            maximum_work_units: definition.maximum_work_units(),
        },
    };
    CustomInvariantRegistration::new(rule).map_err(|error| {
        WorthQueryDomainInstallationDenial::new(
            WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed,
            format!("{}: {error:?}", definition.name().as_str()),
        )
    })
}

#[path = "invariant/execution.rs"]
mod execution;

#[cfg(test)]
#[path = "invariant/lease_tests.rs"]
mod lease_tests;
