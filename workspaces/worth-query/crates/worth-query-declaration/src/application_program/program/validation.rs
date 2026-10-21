//! Validation and canonical construction of one authored program.
use super::*;

pub(super) fn validate<Schema, Program>(
    connections: Vec<ApplicationConnectionDeclaration>,
    rules: Vec<ApplicationProgramRuleDeclaration>,
    operation_bodies: Vec<(
        String,
        crate::application_schema::ApplicationOperationBodyProgram,
    )>,
) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    require_identity(Program::IDENTITY.as_str())?;
    let mut features = Vec::new();
    let mut actions = Vec::new();
    for spec in Program::feature_specs() {
        let (feature, spec_actions) = spec.into_parts();
        features.push(feature);
        actions.extend(spec_actions);
    }
    let feature_ids = validate_features_and_actions(&features, &actions)?;
    delivery_validation::validate_action_deliveries::<Schema, Program>(&actions)?;
    let mut rule_ids = BTreeSet::new();
    for rule in &rules {
        require_identity(rule.composition_instance())?;
        require_identity(rule.identity())?;
        if !rule_ids.insert((
            rule.composition_instance(),
            rule.identity(),
            rule.major(),
            rule.minor(),
            rule.execution_point(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateRule,
                rule.identity(),
            ));
        }
        let scope_exists = features
            .iter()
            .any(|feature| feature.composition_instance() == rule.composition_instance());
        let owner_exists = rule
            .local_owner()
            .is_none_or(|owner| feature_ids.contains(&(rule.composition_instance(), owner)));
        if !scope_exists || !owner_exists {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                rule.identity(),
            ));
        }
    }
    let mut connection_ids = BTreeSet::new();
    let mut bound_inputs = BTreeSet::new();
    for connection in &connections {
        require_identity(connection.source_instance())?;
        require_identity(connection.target_instance())?;
        require_identity(connection.identity())?;
        require_identity(connection.source_port())?;
        require_identity(connection.target_port())?;
        if !connection_ids.insert((
            connection.source_instance(),
            connection.target_instance(),
            connection.identity(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateConnection,
                connection.identity(),
            ));
        }
        if !feature_ids.contains(&(connection.source_instance(), connection.source_feature()))
            || !feature_ids.contains(&(connection.target_instance(), connection.target_feature()))
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                connection.identity(),
            ));
        }
        let source = features
            .iter()
            .find(|feature| {
                feature.composition_instance() == connection.source_instance()
                    && feature.identity() == connection.source_feature()
            })
            .expect("the source feature was proven present");
        if !source
            .outputs()
            .iter()
            .any(|output| output.identity() == connection.source_port())
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UndeclaredOutput,
                format!(
                    "{}.{}",
                    connection.source_feature(),
                    connection.source_port()
                ),
            ));
        }
        if connection.source_instance() != connection.target_instance()
            && !connection.exports_across_instances()
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UnexportedCrossInstanceConnection,
                connection.identity(),
            ));
        }
        let target = features
            .iter()
            .find(|feature| {
                feature.composition_instance() == connection.target_instance()
                    && feature.identity() == connection.target_feature()
            })
            .expect("the target feature was proven present");
        if Program::DERIVED_ARTIFACT_GOVERNANCE
            == super::super::ApplicationDerivedArtifactGovernance::Required
            && target.derived_artifacts().is_empty()
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UngovernedDerivedOutput,
                format!(
                    "{}.{}",
                    connection.target_instance(),
                    connection.target_feature()
                ),
            ));
        }
        if !target.inputs().iter().any(|input| {
            input.identity() == connection.target_port()
                && input.required() == connection.target_required()
        }) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UndeclaredInput,
                format!(
                    "{}.{}",
                    connection.target_feature(),
                    connection.target_port()
                ),
            ));
        }
        if !bound_inputs.insert((
            connection.target_instance(),
            connection.target_feature(),
            connection.target_port(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateInputBinding,
                connection.target_port(),
            ));
        }
    }
    require_acyclic_connections(&features, &connections)?;
    for feature in &features {
        for input in feature.required_inputs() {
            if !bound_inputs.contains(&(feature.composition_instance(), feature.identity(), input))
            {
                return Err(denial(
                    ApplicationProgramValidationDenialKind::MissingRequiredInput,
                    format!("{}.{input}", feature.identity()),
                ));
            }
        }
    }
    body_binding::bind(&mut actions, &operation_bodies);
    let identity = Program::IDENTITY.clone();
    let features = features.into_boxed_slice();
    let actions = actions.into_boxed_slice();
    let connections = connections.into_boxed_slice();
    let rules = rules.into_boxed_slice();
    let adoption_authorization =
        Program::adoption_authorization(super::super::adoption_authorization::DeclarationVisitor);
    if let Some(authorization) = adoption_authorization.as_ref() {
        require_identity(authorization.binding_identity())?;
    }
    let manifest = ApplicationProgramManifest::normalize(
        identity.as_str(),
        &features,
        &actions,
        &connections,
        &rules,
        adoption_authorization.as_ref(),
    );
    let revision = ApplicationProgramRevision::mint(&manifest, Program::CANONICAL_REVISION_BUDGET)
        .map_err(|budget| deny_canonical_revision_budget(&identity, budget))?;
    let semantic_description = ApplicationSemanticDescription::from_validated_parts(
        revision,
        &features,
        &actions,
        &connections,
        &rules,
        adoption_authorization.as_ref(),
    );
    Ok(ValidatedApplicationProgram {
        identity,
        revision,
        features,
        actions,
        connections,
        rules,
        semantic_description,
        adoption_authorization,
        marker: PhantomData,
    })
}
