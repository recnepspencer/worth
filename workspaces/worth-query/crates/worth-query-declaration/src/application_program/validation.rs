use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationFeatureDeclaration, ApplicationFeaturePosture, ApplicationProgramConnectionRole,
    ApplicationProgramConnectionSet, ApplicationProgramDefinition, ApplicationProgramFeatureSet,
    ApplicationProgramInventorySet, ApplicationProgramRuleSet, ApplicationProgramValidationDenial,
    ApplicationProgramValidationDenialKind as Kind, ValidatedApplicationProgram,
};

pub(super) fn validate<Schema, Program>(
) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    require_identity(Program::IDENTITY.as_str())?;
    let features = Program::Features::declarations();
    let connections = Program::Connections::declarations();
    let rules = Program::Rules::declarations();
    let inventories = Program::Inventories::declarations();
    if features.is_empty() || inventories.is_empty() {
        return Err(denial(Kind::EmptyProgram, Program::IDENTITY.as_str()));
    }
    validate_features(&features)?;
    validate_connections(&features, &connections)?;
    validate_rules(&features, &rules)?;
    validate_inventories(&features, &connections, &rules, &inventories)?;
    reject_cycles(&features, &connections)?;
    Ok(ValidatedApplicationProgram::from_parts(
        Program::IDENTITY.clone(),
        features,
        connections,
        rules,
        inventories,
    ))
}

fn validate_features(
    features: &[ApplicationFeatureDeclaration],
) -> Result<(), ApplicationProgramValidationDenial> {
    let mut identities = BTreeSet::new();
    for feature in features {
        require_identity(feature.identity())?;
        if !identities.insert(feature.identity()) {
            return Err(denial(Kind::DuplicateFeature, feature.identity()));
        }
        let mut ports = BTreeSet::new();
        for port in feature.inputs().iter().chain(feature.outputs()) {
            require_identity(port.identity())?;
            if !ports.insert(port.identity()) {
                return Err(denial(
                    Kind::DuplicatePort,
                    format!("{}.{}", feature.identity(), port.identity()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_connections(
    features: &[ApplicationFeatureDeclaration],
    connections: &[super::ApplicationConnectionDeclaration],
) -> Result<(), ApplicationProgramValidationDenial> {
    let by_id = features
        .iter()
        .map(|feature| (feature.identity(), feature))
        .collect::<BTreeMap<_, _>>();
    let mut connection_ids = BTreeSet::new();
    let mut bound_inputs = BTreeSet::new();
    let mut incoming_features = BTreeSet::new();
    for connection in connections {
        require_identity(connection.identity())?;
        if !connection_ids.insert(connection.identity()) {
            return Err(denial(Kind::DuplicateConnection, connection.identity()));
        }
        let Some(source) = by_id.get(connection.source_feature()) else {
            return Err(denial(Kind::DanglingFeature, connection.identity()));
        };
        let Some(target) = by_id.get(connection.target_feature()) else {
            return Err(denial(Kind::DanglingFeature, connection.identity()));
        };
        if source.type_id() != connection.source_feature_type()
            || target.type_id() != connection.target_feature_type()
        {
            return Err(denial(Kind::ForeignFeatureType, connection.identity()));
        }
        let target_available = target.posture() == ApplicationFeaturePosture::Available;
        let source_available = source.posture() == ApplicationFeaturePosture::Available;
        let binding_available = connection.role() != ApplicationProgramConnectionRole::Unavailable;
        if target_available != binding_available || (!source_available && target_available) {
            return Err(denial(Kind::AvailabilityMismatch, connection.identity()));
        }
        if !incoming_features.insert(connection.target_feature()) {
            return Err(denial(Kind::UnsupportedFanIn, connection.target_feature()));
        }
        let Some(source_port) = source
            .outputs()
            .iter()
            .find(|port| port.identity() == connection.source_port())
        else {
            return Err(denial(Kind::DanglingPort, connection.identity()));
        };
        if source_port.type_id() != connection.source_port_type() {
            return Err(denial(Kind::ForeignPortType, connection.identity()));
        }
        let Some(target_port) = target
            .inputs()
            .iter()
            .find(|port| port.identity() == connection.target_port())
        else {
            return Err(denial(Kind::DanglingPort, connection.identity()));
        };
        if target_port.type_id() != connection.target_port_type() {
            return Err(denial(Kind::ForeignPortType, connection.identity()));
        }
        let input = (connection.target_feature(), connection.target_port());
        if !bound_inputs.insert(input) {
            return Err(denial(
                Kind::DuplicateInputBinding,
                format!("{}.{}", input.0, input.1),
            ));
        }
    }
    for feature in features {
        for input in feature.inputs().iter().filter(|input| input.required()) {
            if !bound_inputs.contains(&(feature.identity(), input.identity())) {
                return Err(denial(
                    Kind::MissingRequiredInput,
                    format!("{}.{}", feature.identity(), input.identity()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_rules(
    features: &[ApplicationFeatureDeclaration],
    rules: &[super::ApplicationProgramRuleDeclaration],
) -> Result<(), ApplicationProgramValidationDenial> {
    let feature_postures = features
        .iter()
        .map(|feature| (feature.identity(), feature.posture()))
        .collect::<BTreeMap<_, _>>();
    let mut rule_ids = BTreeSet::new();
    for rule in rules {
        require_identity(rule.identity())?;
        let key = (
            rule.identity(),
            rule.major(),
            rule.minor(),
            rule.execution_point(),
            rule.local_owner(),
        );
        if !rule_ids.insert(key) {
            return Err(denial(Kind::DuplicateRule, rule.identity()));
        }
        if rule
            .local_owner()
            .is_some_and(|owner| !feature_postures.contains_key(owner))
        {
            return Err(denial(Kind::DanglingFeature, rule.identity()));
        }
        if rule.local_owner().is_some_and(|owner| {
            rule.posture() == super::ApplicationProgramRulePosture::Available
                && feature_postures.get(owner) == Some(&ApplicationFeaturePosture::Unavailable)
        }) {
            return Err(denial(Kind::AvailabilityMismatch, rule.identity()));
        }
    }
    Ok(())
}

fn validate_inventories(
    features: &[ApplicationFeatureDeclaration],
    connections: &[super::ApplicationConnectionDeclaration],
    rules: &[super::ApplicationProgramRuleDeclaration],
    inventories: &[super::ApplicationProgramInventoryDeclaration],
) -> Result<(), ApplicationProgramValidationDenial> {
    let outputs = features
        .iter()
        .flat_map(|feature| {
            feature.outputs().iter().map(move |port| {
                (
                    (feature.identity(), port.identity()),
                    (
                        feature.type_id(),
                        port.type_id(),
                        port.required(),
                        feature.posture(),
                    ),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let mut inventory_ids = BTreeSet::new();
    let mut covered = BTreeSet::new();
    let mut inventory_features = BTreeSet::new();
    for inventory in inventories {
        require_identity(inventory.identity())?;
        if !inventory_ids.insert(inventory.identity()) {
            return Err(denial(Kind::DuplicateInventory, inventory.identity()));
        }
        if inventory.outputs().is_empty() {
            return Err(denial(Kind::EmptyInventory, inventory.identity()));
        }
        let mut members = BTreeSet::new();
        for output in inventory.outputs() {
            let key = (output.feature(), output.port());
            let Some((feature_type, port_type, _, _posture)) = outputs.get(&key) else {
                return Err(denial(
                    Kind::UnknownInventoryOutput,
                    format!("{}:{}.{}", inventory.identity(), key.0, key.1),
                ));
            };
            if *feature_type != output.feature_type() || *port_type != output.port_type() {
                return Err(denial(
                    Kind::UnknownInventoryOutput,
                    format!("{}:{}.{}", inventory.identity(), key.0, key.1),
                ));
            }
            if !members.insert(key) {
                return Err(denial(
                    Kind::DuplicateInventoryOutput,
                    format!("{}:{}.{}", inventory.identity(), key.0, key.1),
                ));
            }
            covered.insert(key);
            inventory_features.insert(key.0);
        }
    }
    for (output, (_, _, required, _)) in outputs {
        if required && !covered.contains(&output) {
            return Err(denial(
                Kind::IncompleteInventory,
                format!("{}.{}", output.0, output.1),
            ));
        }
    }
    let mut closure = inventory_features;
    closure.extend(
        rules
            .iter()
            .filter(|rule| rule.posture() == super::ApplicationProgramRulePosture::Available)
            .filter_map(|rule| rule.local_owner()),
    );
    loop {
        let before = closure.len();
        for connection in connections {
            if closure.contains(connection.target_feature()) {
                closure.insert(connection.source_feature());
            }
        }
        if closure.len() == before {
            break;
        }
    }
    if let Some(orphan) = features.iter().find(|feature| {
        feature.posture() == ApplicationFeaturePosture::Available
            && !closure.contains(feature.identity())
    }) {
        return Err(denial(Kind::OrphanFeature, orphan.identity()));
    }
    Ok(())
}

fn reject_cycles(
    features: &[ApplicationFeatureDeclaration],
    connections: &[super::ApplicationConnectionDeclaration],
) -> Result<(), ApplicationProgramValidationDenial> {
    let mut indegree = features
        .iter()
        .map(|feature| (feature.identity(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<&str, Vec<&str>>::new();
    for connection in connections {
        *indegree.entry(connection.target_feature()).or_default() += 1;
        outgoing
            .entry(connection.source_feature())
            .or_default()
            .push(connection.target_feature());
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(feature, count)| (*count == 0).then_some(*feature))
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    while let Some(feature) = ready.pop_front() {
        visited += 1;
        for target in outgoing.get(feature).into_iter().flatten() {
            let count = indegree
                .get_mut(target)
                .expect("validated connection target exists");
            *count -= 1;
            if *count == 0 {
                ready.push_back(target);
            }
        }
    }
    if visited != features.len() {
        let subject = indegree
            .into_iter()
            .find_map(|(feature, count)| (count > 0).then_some(feature))
            .unwrap_or("unknown");
        return Err(denial(Kind::Cycle, subject));
    }
    Ok(())
}

fn require_identity(identity: &str) -> Result<(), ApplicationProgramValidationDenial> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(denial(Kind::InvalidIdentity, identity))
    } else {
        Ok(())
    }
}

fn denial(kind: Kind, subject: impl Into<String>) -> ApplicationProgramValidationDenial {
    ApplicationProgramValidationDenial::new(kind, subject)
}
