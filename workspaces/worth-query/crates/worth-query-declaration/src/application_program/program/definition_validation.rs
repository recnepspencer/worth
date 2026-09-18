use std::collections::BTreeSet;

use super::{
    denial, require_identity, ApplicationActionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};

pub(super) type FeatureIdentity = (&'static str, &'static str);

pub(super) fn validate_features_and_actions(
    features: &[ApplicationFeatureDeclaration],
    actions: &[ApplicationActionDeclaration],
) -> Result<BTreeSet<FeatureIdentity>, ApplicationProgramValidationDenial> {
    let mut feature_ids = BTreeSet::new();
    for feature in features {
        validate_feature(feature, &mut feature_ids)?;
    }
    let mut action_ids = BTreeSet::new();
    for action in actions {
        require_identity(action.composition_instance())?;
        require_identity(action.feature())?;
        require_identity(action.binding())?;
        if !feature_ids.contains(&(action.composition_instance(), action.feature())) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                action.feature(),
            ));
        }
        if action.locality().is_some() != action.change_shape().is_some() {
            return Err(denial(
                ApplicationProgramValidationDenialKind::IncompleteActionChangeContract,
                action.binding(),
            ));
        }
        if let Some(locality) = action.locality() {
            require_identity(locality.identity())?;
        }
        if let Some(shape) = action.change_shape() {
            require_identity(shape.identity())?;
        }
        if !action_ids.insert((
            action.composition_instance(),
            action.feature(),
            action.action_type(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateAction,
                action.binding(),
            ));
        }
    }
    Ok(feature_ids)
}

fn validate_feature(
    feature: &ApplicationFeatureDeclaration,
    feature_ids: &mut BTreeSet<FeatureIdentity>,
) -> Result<(), ApplicationProgramValidationDenial> {
    require_identity(feature.composition_instance())?;
    require_identity(feature.identity())?;
    if !feature_ids.insert((feature.composition_instance(), feature.identity())) {
        return Err(denial(
            ApplicationProgramValidationDenialKind::DuplicateFeature,
            feature.identity(),
        ));
    }
    let mut input_ids = BTreeSet::new();
    for input in feature.inputs() {
        require_identity(input.identity())?;
        if !input_ids.insert(input.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateInputBinding,
                format!("{}.{}", feature.identity(), input.identity()),
            ));
        }
    }
    let mut output_ids = BTreeSet::new();
    for output in feature.outputs() {
        require_identity(output.identity())?;
        if !output_ids.insert(output.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateOutput,
                format!("{}.{}", feature.identity(), output.identity()),
            ));
        }
    }
    let mut artifact_ids = BTreeSet::new();
    let mut artifact_producers = BTreeSet::new();
    for artifact in feature.derived_artifacts() {
        require_identity(artifact.identity())?;
        require_identity(artifact.locality().identity())?;
        require_identity(artifact.producer_family())?;
        require_identity(artifact.reuse_rule())?;
        require_identity(artifact.stopped_outcome())?;
        for dependency in artifact.dependencies() {
            require_identity(dependency.identity())?;
        }
        if !artifact_ids.insert(artifact.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateDerivedArtifact,
                format!("{}.{}", feature.identity(), artifact.identity()),
            ));
        }
        if !artifact_producers.insert(artifact.producer_family()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::AmbiguousDerivedArtifactProducer,
                format!("{}.{}", feature.identity(), artifact.producer_family()),
            ));
        }
        if !output_ids.contains(artifact.output()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UndeclaredArtifactOutput,
                format!("{}.{}", feature.identity(), artifact.output()),
            ));
        }
    }
    let artifact_types = feature
        .derived_artifacts()
        .iter()
        .map(|artifact| artifact.artifact_type())
        .collect::<BTreeSet<_>>();
    let mut computation_ids = BTreeSet::new();
    for computation in feature.managed_computations() {
        require_identity(computation.identity())?;
        require_identity(computation.input())?;
        require_identity(computation.output_artifact())?;
        require_identity(computation.partition())?;
        require_identity(computation.reuse())?;
        require_identity(computation.stopped())?;
        require_identity(computation.ordering())?;
        if !computation_ids.insert(computation.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateManagedComputation,
                format!("{}.{}", feature.identity(), computation.identity()),
            ));
        }
        if !artifact_types.contains(&computation.output_artifact_type()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::MissingManagedComputationArtifact,
                format!("{}.{}", feature.identity(), computation.output_artifact()),
            ));
        }
        let resources = computation.resources();
        if resources.maximum_work() == 0 || resources.maximum_retained_bytes() == 0 {
            return Err(denial(
                ApplicationProgramValidationDenialKind::InvalidManagedComputationResources,
                computation.identity(),
            ));
        }
    }
    let mut collection_ids = BTreeSet::new();
    for collection in feature.derived_collections() {
        require_identity(collection.identity())?;
        require_identity(collection.contributor())?;
        require_identity(collection.grouping())?;
        require_identity(collection.measures())?;
        require_identity(collection.lineage())?;
        require_identity(collection.applicability())?;
        require_identity(collection.incremental_update())?;
        if !collection_ids.insert(collection.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateDerivedCollection,
                format!("{}.{}", feature.identity(), collection.identity()),
            ));
        }
    }
    Ok(())
}
