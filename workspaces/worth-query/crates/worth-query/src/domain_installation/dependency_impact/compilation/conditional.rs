use crate::domain_installation::WorthQueryConditionalProvenance;

use super::super::compiled::{
    WorthQueryCompiledSemanticAspectDependency, WorthQueryConditionalObservationEvidence,
    WorthQuerySemanticAspectDependencyLocus, WorthQuerySemanticAspectDependencySource,
    WorthQuerySemanticDependencyRole,
};
use super::operation_definition::SemanticAspectDependencyCompilation;
use super::WorthQuerySemanticAspectDependencyCompilationDenialKind;

impl SemanticAspectDependencyCompilation {
    pub(super) fn push_conditional(
        &mut self,
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        declaration: &worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
    ) {
        self.conditional_declarations
            .insert(location.clone(), declaration.clone());
        self.conditional_order.push(location.clone());
        self.dependencies
            .push(WorthQueryCompiledSemanticAspectDependency::new(
                WorthQuerySemanticAspectDependencyLocus::ConditionalNode {
                    location: location.clone(),
                },
                WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness,
                WorthQuerySemanticAspectDependencySource::ConditionalNodeContract(
                    declaration.clone(),
                ),
            ));
        self.counters.conditional_node_visits += 1;
        for (dependency_ordinal, dependency) in declaration.dependencies().iter().enumerate() {
            self.dependencies
                .push(WorthQueryCompiledSemanticAspectDependency::new(
                    WorthQuerySemanticAspectDependencyLocus::ConditionalTruth {
                        location: location.clone(),
                        dependency_ordinal,
                    },
                    WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness,
                    WorthQuerySemanticAspectDependencySource::ConditionalTruth(dependency.clone()),
                ));
            self.counters.conditional_truth_edges += 1;
        }
    }

    pub(super) fn push_realized_conditionals(
        &mut self,
        conditionals: &[&WorthQueryConditionalProvenance],
    ) -> Result<(), WorthQuerySemanticAspectDependencyCompilationDenialKind> {
        let mut by_location = std::collections::HashMap::with_capacity(conditionals.len());
        for conditional in conditionals {
            if by_location
                .insert(conditional.location().clone(), *conditional)
                .is_some()
            {
                return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalDeclarationMismatch);
            }
        }
        if by_location.len() != self.conditional_order.len() {
            return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalDeclarationMismatch);
        }
        let canonical_order = self.conditional_order.clone();
        for location in canonical_order {
            if let Some(conditional) = by_location.remove(&location) {
                self.push_realized_conditional(conditional)?;
            }
        }
        if !by_location.is_empty() {
            return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalDeclarationMismatch);
        }
        Ok(())
    }

    fn push_realized_conditional(
        &mut self,
        conditional: &WorthQueryConditionalProvenance,
    ) -> Result<(), WorthQuerySemanticAspectDependencyCompilationDenialKind> {
        let Some(installed) = self.conditional_declarations.get(conditional.location()) else {
            return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalDeclarationMismatch);
        };
        if installed != conditional.declaration() {
            return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalDeclarationMismatch);
        }
        let dependency_count = installed.dependencies().len();
        use worth_query_installation::facade::WorthQueryConditionalConditionClass as ConditionClass;
        let plan_bearing = matches!(
            installed.condition().class(),
            ConditionClass::DeltaThreshold
                | ConditionClass::Temporal
                | ConditionClass::DomainSpecific
        );
        let expected_observations =
            if plan_bearing && conditional.bridge.bridge_snapshot_identity().is_some() {
                let condition_dependencies = installed.condition().dependencies();
                installed
                    .dependencies()
                    .iter()
                    .enumerate()
                    .filter_map(|(ordinal, dependency)| {
                        (condition_dependencies.is_empty()
                            || condition_dependencies.contains(dependency))
                        .then_some(ordinal)
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
        let mut observed =
            std::collections::HashMap::with_capacity(conditional.semantic_observation_count());
        for ordinal in 0..dependency_count {
            if let Some(observation) = conditional.semantic_observation(ordinal)
                && observed.insert(ordinal, observation).is_some()
            {
                return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalObservationMismatch);
            }
        }
        if observed.len() != conditional.semantic_observation_count() {
            return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalObservationMismatch);
        }
        let mut observed_ordinals = observed.keys().copied().collect::<Vec<_>>();
        observed_ordinals.sort_unstable();
        if observed_ordinals != expected_observations {
            return Err(WorthQuerySemanticAspectDependencyCompilationDenialKind::RealizedConditionalObservationMismatch);
        }
        let observations = (0..dependency_count)
            .map(|dependency_ordinal| {
                let observation = observed.remove(&dependency_ordinal);
                WorthQueryConditionalObservationEvidence {
                    dependency_ordinal,
                    previous: observation
                        .as_ref()
                        .and_then(|item| item.previous().cloned()),
                    current: observation.and_then(|item| item.current().cloned()),
                }
            })
            .collect::<Vec<_>>();
        self.counters.conditional_observations_retained += observations
            .iter()
            .filter(|item| item.was_observed())
            .count();
        self.dependencies
            .push(WorthQueryCompiledSemanticAspectDependency::new(
                WorthQuerySemanticAspectDependencyLocus::ConditionalOutcome {
                    location: conditional.location().clone(),
                },
                WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness,
                WorthQuerySemanticAspectDependencySource::RealizedConditionalOutcome {
                    class: conditional.class(),
                    signal_projection: conditional.signal_projection().clone(),
                    observations,
                },
            ));
        self.counters.realized_conditional_outcome_edges += 1;
        Ok(())
    }
}
