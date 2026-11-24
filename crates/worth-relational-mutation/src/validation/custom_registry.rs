use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contracts::validation::data::execution::InvariantExecutionPoint;
use crate::contracts::validation::data::rule_id::CustomInvariantSemanticIdentity;
use crate::validation::candidate_source::SourcePreparationGroup;
use crate::validation::custom_rule::CustomInvariantRegistration;
pub mod compilation;
pub mod compilation_quote;
pub mod source_inventory;
pub use crate::contracts::validation::data::compilation_denial::CustomInvariantCompilationDenial;
pub use source_inventory::{CustomInvariantCompilationFunding, SourcePreparationInventory};

pub use crate::contracts::validation::data::registration_conflict::DuplicateCustomInvariantRegistration;

#[derive(Debug, Clone, Default)]
pub struct FrozenCustomInvariantRegistry {
    registrations: Arc<[CustomInvariantRegistration]>,
    source_groups: Arc<SourcePreparationInventory>,
    source_postings: Arc<crate::validation::candidate_source::routing::SourceProgramPostings>,
    points: Arc<[Vec<usize>; InvariantExecutionPoint::COUNT]>,
    model_ownership_index: Option<usize>,
    pub source_history:
        Arc<worth_relational_branch::internal_port::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog>,
    index_by_identity_and_execution_point: Arc<
        BTreeMap<
            (
                crate::contracts::validation::data::rule_id::CustomInvariantRuleId,
                InvariantExecutionPoint,
            ),
            usize,
        >,
    >,
    _funding: Option<Arc<CustomInvariantCompilationFunding>>,
}

impl FrozenCustomInvariantRegistry {
    pub fn registrations(&self) -> &[CustomInvariantRegistration] {
        &self.registrations
    }

    /// The exact tagged registration; no request-time applicability scan or parallel meaning.
    pub fn model_ownership(
        &self,
    ) -> Option<(
        &CustomInvariantRegistration,
        &crate::branch::model_ownership::RelationalModelOwnership,
    )> {
        let registration = self.registrations.get(self.model_ownership_index?)?;
        Some((registration, registration.model_ownership.as_deref()?))
    }

    pub fn source_inventory(&self) -> Arc<SourcePreparationInventory> {
        self.source_groups.clone()
    }

    pub fn all_source_groups(&self) -> &[SourcePreparationGroup] {
        &self.source_groups
    }

    pub fn explicit_source_programs(
        &self,
    ) -> Vec<crate::validation::candidate_source::ManagedIndexSourceProgram> {
        self.source_groups
            .iter()
            .flat_map(SourcePreparationGroup::public_programs)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    pub(crate) fn source_groups(
        &self,
        entities: impl Iterator<Item = crate::contracts::identity::data::primitives::KindId>,
        relations: impl Iterator<Item = crate::contracts::identity::data::primitives::KindId>,
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
    pub fn iter(&self) -> impl Iterator<Item = &CustomInvariantRegistration> {
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
pub mod tests {
    use std::sync::Arc;

    use crate::contracts::validation::data::custom_rule::errors::CustomInvariantExecutionError;
    use crate::contracts::validation::data::custom_rule::errors::CustomInvariantPreparationError;
    use crate::contracts::validation::data::custom_rule::errors::CustomInvariantVerdict;
    use crate::contracts::validation::data::descriptor::CustomInvariantDescriptor;
    use crate::contracts::validation::data::descriptor::CustomInvariantOperationalMetadata;
    use crate::contracts::validation::data::execution::InvariantExecutionPoint;
    use crate::contracts::validation::data::execution::InvariantFailureEffect;
    use crate::contracts::validation::data::groups::InvariantCostClass;
    use crate::contracts::validation::data::groups::InvariantGroup;
    use crate::contracts::validation::data::groups::InvariantGroupSet;
    use crate::contracts::validation::data::rule_id::CustomInvariantRuleId;
    use crate::contracts::validation::data::rule_id::CustomInvariantSemanticIdentity;
    use crate::contracts::validation::data::rule_id::CustomInvariantSemanticVersion;
    use crate::validation::custom_registry::FrozenCustomInvariantRegistry;
    use crate::validation::custom_rule::CustomInvariantExecutionContext;
    use crate::validation::custom_rule::CustomInvariantRegistration;
    use crate::validation::custom_rule::CustomInvariantRule;
    use crate::validation::custom_rule::CustomInvariantScopePlanner;

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
                    access:
                        crate::contracts::validation::data::descriptor::CustomInvariantAccessContract::default(
                        ),
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

        let error = FrozenCustomInvariantRegistry::compile(
            &crate::contracts::schema::data::RelationalSchemaRegistry::default(),
            vec![left, right],
            Vec::new(),
            crate::contracts::schema::compilation_control::SchemaCompilationControl::Configuration,
        )
        .unwrap_err();
        let crate::contracts::validation::data::compilation_denial::CustomInvariantCompilationDenial::Duplicate(error) = error else {
            panic!("duplicate semantics must remain typed");
        };
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

        let registry = FrozenCustomInvariantRegistry::compile(
            &crate::contracts::schema::data::RelationalSchemaRegistry::default(),
            vec![commit_backstop, graph_composition],
            Vec::new(),
            crate::contracts::schema::compilation_control::SchemaCompilationControl::Configuration,
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
        let registry = FrozenCustomInvariantRegistry::compile(
            &crate::contracts::schema::data::RelationalSchemaRegistry::default(),
            vec![registration],
            Vec::new(),
            crate::contracts::schema::compilation_control::SchemaCompilationControl::Configuration,
        )
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
