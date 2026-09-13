use std::collections::BTreeMap;
use std::sync::Arc;

use crate::validation::data::{
    CustomInvariantRegistration, CustomInvariantSemanticIdentity, InvariantExecutionPoint,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DuplicateCustomInvariantRegistration {
    pub(crate) identity: CustomInvariantSemanticIdentity,
    pub(crate) execution_point: InvariantExecutionPoint,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FrozenCustomInvariantRegistry {
    registrations: Arc<[CustomInvariantRegistration]>,
    index_by_identity_and_execution_point: Arc<
        BTreeMap<
            (
                crate::validation::data::CustomInvariantRuleId,
                InvariantExecutionPoint,
            ),
            usize,
        >,
    >,
}

impl FrozenCustomInvariantRegistry {
    pub(crate) fn from_registrations(
        registrations: Vec<CustomInvariantRegistration>,
    ) -> Result<Self, DuplicateCustomInvariantRegistration> {
        let mut index_by_identity_and_execution_point = BTreeMap::new();
        for (index, registration) in registrations.iter().enumerate() {
            let identity = registration.descriptor().identity.clone();
            let execution_point = registration.execution_point();
            if index_by_identity_and_execution_point
                .insert((identity.rule_id.clone(), execution_point), index)
                .is_some()
            {
                return Err(DuplicateCustomInvariantRegistration {
                    identity,
                    execution_point,
                });
            }
        }
        Ok(Self {
            registrations: registrations.into(),
            index_by_identity_and_execution_point: Arc::new(index_by_identity_and_execution_point),
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.registrations.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &CustomInvariantRegistration> {
        self.registrations.iter()
    }

    pub(crate) fn get(
        &self,
        identity: &CustomInvariantSemanticIdentity,
        execution_point: InvariantExecutionPoint,
    ) -> Option<&CustomInvariantRegistration> {
        self.index_by_identity_and_execution_point
            .get(&(identity.rule_id.clone(), execution_point))
            .and_then(|index| self.registrations.get(*index))
            .filter(|registration| &registration.descriptor().identity == identity)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::FrozenCustomInvariantRegistry;
    use crate::validation::data::{
        CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
        CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
        CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
        CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
        CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
        InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
    };

    #[derive(Clone, Copy)]
    struct RegistryRule {
        rule_id: &'static str,
        execution_point: InvariantExecutionPoint,
    }

    impl CustomInvariantRule for RegistryRule {
        type Scope = ();

        fn descriptor(&self) -> CustomInvariantDescriptor {
            CustomInvariantDescriptor {
                identity: CustomInvariantSemanticIdentity {
                    rule_id: CustomInvariantRuleId::new(self.rule_id),
                    semantic_version: CustomInvariantSemanticVersion::new(1, 0),
                },
                display_name: Arc::from(self.rule_id),
                operational: CustomInvariantOperationalMetadata {
                    maximum_work_units: std::num::NonZeroU64::new(1).unwrap(),
                    access: crate::validation::data::CustomInvariantAccessContract::default(),
                    execution_point: self.execution_point,
                    groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                    cost_class: InvariantCostClass::Touched,
                    failure_effect: InvariantFailureEffect::BlockCommit,
                },
            }
        }

        fn prepare_scope(
            &self,
            _planner: &mut CustomInvariantScopePlanner<'_>,
        ) -> Result<Self::Scope, CustomInvariantPreparationError> {
            Ok(())
        }

        fn evaluate(
            &self,
            _context: &CustomInvariantExecutionContext<'_>,
            _scope: &Self::Scope,
        ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
            Ok(CustomInvariantVerdict::Pass)
        }
    }

    #[test]
    fn frozen_registry_rejects_duplicate_semantic_identity_at_same_execution_point() {
        let left = CustomInvariantRegistration::new(registry_rule(
            "dup.rule",
            InvariantExecutionPoint::CommitBoundary,
        ))
        .unwrap();
        let right = CustomInvariantRegistration::new(registry_rule(
            "dup.rule",
            InvariantExecutionPoint::CommitBoundary,
        ))
        .unwrap();

        let error =
            FrozenCustomInvariantRegistry::from_registrations(vec![left, right]).unwrap_err();
        assert_eq!(error.identity.rule_id.as_str(), "dup.rule");
        assert_eq!(
            error.execution_point,
            InvariantExecutionPoint::CommitBoundary
        );
    }

    #[test]
    fn frozen_registry_allows_one_semantic_identity_at_distinct_execution_points() {
        let commit_backstop = CustomInvariantRegistration::new(registry_rule(
            "paired.rule",
            InvariantExecutionPoint::CommitBoundary,
        ))
        .unwrap();
        let graph_composition = CustomInvariantRegistration::new(registry_rule(
            "paired.rule",
            InvariantExecutionPoint::GraphComposition,
        ))
        .unwrap();
        let identity = commit_backstop.descriptor().identity.clone();

        let registry = FrozenCustomInvariantRegistry::from_registrations(vec![
            commit_backstop,
            graph_composition,
        ])
        .unwrap();

        assert_eq!(registry.len(), 2);
        assert!(registry
            .get(&identity, InvariantExecutionPoint::CommitBoundary)
            .is_some());
        assert!(registry
            .get(&identity, InvariantExecutionPoint::GraphComposition)
            .is_some());
    }

    #[test]
    fn frozen_registry_supports_stable_lookup() {
        let registration = CustomInvariantRegistration::new(registry_rule(
            "lookup.rule",
            InvariantExecutionPoint::CommitBoundary,
        ))
        .unwrap();
        let identity = registration.descriptor().identity.clone();
        let registry =
            FrozenCustomInvariantRegistry::from_registrations(vec![registration]).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry
            .get(&identity, InvariantExecutionPoint::CommitBoundary)
            .is_some());
    }

    fn registry_rule(
        rule_id: &'static str,
        execution_point: InvariantExecutionPoint,
    ) -> RegistryRule {
        RegistryRule {
            rule_id,
            execution_point,
        }
    }
}
