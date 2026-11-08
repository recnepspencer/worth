use std::collections::BTreeMap;
use std::sync::Arc;

use crate::validation::candidate_source::SourcePreparationGroup;
use crate::validation::data::{
    CustomInvariantRegistration, CustomInvariantSemanticIdentity, InvariantExecutionPoint,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateCustomInvariantRegistration {
    pub identity: CustomInvariantSemanticIdentity,
    pub execution_point: InvariantExecutionPoint,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FrozenCustomInvariantRegistry {
    registrations: Arc<[CustomInvariantRegistration]>,
    points: Arc<[Vec<usize>; InvariantExecutionPoint::COUNT]>,
    source_groups: Arc<[SourcePreparationGroup]>,
    source_postings: crate::validation::candidate_source::routing::SourceProgramPostings,
    pub(crate) source_history:
        Arc<crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog>,
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
        public_sources: Vec<SourcePreparationGroup>,
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
        let mut source_groups = registrations
            .iter()
            .filter_map(SourcePreparationGroup::from_invariant)
            .collect::<Vec<_>>();
        source_groups.extend(public_sources);
        Ok(Self {
            source_history: Arc::new(crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog::compile(&source_groups)),
            source_postings:
                crate::validation::candidate_source::routing::SourceProgramPostings::compile(
                    &source_groups,
                ),
            source_groups: source_groups.into(),
            points: Arc::new(std::array::from_fn(|point| registrations.iter().enumerate().filter(|(_, r)| point == r.execution_point() as usize || (point == InvariantExecutionPoint::ProposedState as usize && InvariantExecutionPoint::ProposedState.includes(r.execution_point()))).map(|(i,_)| i).collect())),
            registrations: registrations.into(),
            index_by_identity_and_execution_point: Arc::new(index_by_identity_and_execution_point),
        })
    }

    pub(crate) fn source_inventory(&self) -> Arc<[SourcePreparationGroup]> {
        self.source_groups.clone()
    }

    pub(crate) fn explicit_source_programs(
        &self,
    ) -> Vec<crate::validation::candidate_source::ManagedIndexSourceProgram> {
        self.source_groups
            .iter()
            .flat_map(SourcePreparationGroup::public_programs)
            .cloned()
            .collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.registrations.len()
    }

    pub(crate) fn source_groups(
        &self,
        entities: impl Iterator<Item = crate::identity::data::KindId>,
        relations: impl Iterator<Item = crate::identity::data::KindId>,
    ) -> Vec<&SourcePreparationGroup> {
        self.source_postings
            .affected(entities, relations)
            .into_iter()
            .map(|slot| &self.source_groups[slot])
            .collect()
    }

    pub(crate) fn at(
        &self,
        point: InvariantExecutionPoint,
    ) -> impl Iterator<Item = &CustomInvariantRegistration> {
        self.points[point as usize]
            .iter()
            .map(|&i| &self.registrations[i])
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
                    maximum_work_units: Some(std::num::NonZeroU64::new(1).unwrap()),
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
            FrozenCustomInvariantRegistry::from_registrations(vec![left, right], Vec::new())
                .unwrap_err();
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

        let registry = FrozenCustomInvariantRegistry::from_registrations(
            vec![commit_backstop, graph_composition],
            Vec::new(),
        )
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
            FrozenCustomInvariantRegistry::from_registrations(vec![registration], Vec::new())
                .unwrap();

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
