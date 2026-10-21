//! Declared managed computations must have their exact installed owner.
use super::*;

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct InstalledManagedComputationOwner {
    pub feature_type: TypeId,
    pub computation_type: TypeId,
    pub output_artifact_type: TypeId,
    pub(super) collection: Option<super::collections::InstalledCollection>,
}

#[derive(Clone, Copy)]
pub(super) struct ManagedComputationExpectation {
    composition_instance: &'static str,
    feature_identity: &'static str,
    identity: &'static str,
    feature_type: TypeId,
    computation_type: TypeId,
    output_artifact_type: TypeId,
}

impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema> {
    /// Records the one owner of a managed computation. `binding` is the
    /// execution posture the owner's binding serves: the declared posture must
    /// be that one. A declared equivalence predicate is refused because no
    /// installed predicate is visible here.
    pub(in crate::domain_computation::primary_graph) fn record_computation_owner<
        Feature,
        Computation,
    >(
        &mut self,
        binding: ApplicationComputationExecution,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Feature: worth_query_declaration::facade::application_program::ApplicationFeature<Schema>,
        Computation:
            worth_query_declaration::facade::application_program::ApplicationManagedComputation<
                Schema,
                Feature,
            >,
    {
        if Computation::EXECUTION != binding {
            return Err(denial(
                DenialKind::ManagedComputationOwnerBindingMismatch,
                Computation::IDENTITY,
            ));
        }
        if let DeterminismContract::ContractEquivalent(_) = Computation::DETERMINISM {
            return Err(denial(
                DenialKind::ManagedComputationEquivalenceUnavailable,
                Computation::IDENTITY,
            ));
        }
        let computation_type = TypeId::of::<Computation>();
        if self.computations.contains_key(&computation_type) {
            return Err(denial(
                DenialKind::DuplicateManagedComputationOwner,
                Computation::IDENTITY,
            ));
        }
        self.computations.insert(
            computation_type,
            InstalledManagedComputationOwner {
                collection: None,
                feature_type: TypeId::of::<Feature>(),
                computation_type,
                output_artifact_type: TypeId::of::<Computation::Output>(),
            },
        );
        Ok(())
    }
}

impl<Schema> PendingMutationHandlerRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn validate_managed_computations(
        &self,
        features: &[worth_query_declaration::facade::application_program::ApplicationFeatureDeclaration],
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let mut declared = Vec::new();
        for (feature, computation) in features.iter().flat_map(|feature| {
            feature
                .managed_computations()
                .iter()
                .map(move |computation| (feature, computation))
        }) {
            let entry = (
                TypeId::from(computation.computation_type()),
                ManagedComputationExpectation {
                    composition_instance: feature.composition_instance(),
                    feature_identity: feature.identity(),
                    identity: computation.identity(),
                    feature_type: TypeId::from(computation.feature_type()),
                    computation_type: TypeId::from(computation.computation_type()),
                    output_artifact_type: TypeId::from(computation.output_artifact_type()),
                },
            );
            declared.push(entry);
        }
        validate_managed_computation_inventory(&self.computations, &declared)
    }
}

fn validate_managed_computation_inventory(
    installed: &HashMap<TypeId, InstalledManagedComputationOwner>,
    declared: &[(TypeId, ManagedComputationExpectation)],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let mut computations: Vec<_> = declared.iter().collect();
    computations.sort_by_key(|(_, computation)| {
        (
            computation.composition_instance,
            computation.feature_identity,
            computation.identity,
        )
    });
    for (computation_type, computation) in computations {
        let owner = installed.get(computation_type).ok_or_else(|| {
            denial(
                DenialKind::MissingManagedComputationOwner,
                computation.identity,
            )
        })?;
        if computation.computation_type != *computation_type
            || owner.computation_type != *computation_type
            || owner.feature_type != computation.feature_type
            || owner.output_artifact_type != computation.output_artifact_type
        {
            return Err(denial(
                DenialKind::ManagedComputationOwnerMeaningMismatch,
                computation.identity,
            ));
        }
    }
    let unique = declared
        .iter()
        .map(|(identity, _)| *identity)
        .collect::<std::collections::HashSet<_>>();
    if installed.len() != unique.len() {
        return Err(denial(
            DenialKind::ForeignManagedComputationOwner,
            "managed computation owner inventory",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../registry_tests.rs"]
mod tests;
