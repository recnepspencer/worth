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
        if !output_ids.contains(artifact.output()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UndeclaredArtifactOutput,
                format!("{}.{}", feature.identity(), artifact.output()),
            ));
        }
    }
    Ok(())
}
