use super::super::{
    ApplicationActionDeclaration, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramRevision, ApplicationProgramRuleDeclaration,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationSemanticFamily {
    Features,
    Ports,
    Connections,
    Rules,
    Operations,
    ExternalInputs,
    Outputs,
    Resources,
}

impl ApplicationSemanticFamily {
    pub const ALL: [Self; 8] = [
        Self::Features,
        Self::Ports,
        Self::Connections,
        Self::Rules,
        Self::Operations,
        Self::ExternalInputs,
        Self::Outputs,
        Self::Resources,
    ];
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationSemanticFact {
    family: ApplicationSemanticFamily,
    subject: String,
    meaning: String,
}

impl ApplicationSemanticFact {
    pub const fn family(&self) -> ApplicationSemanticFamily {
        self.family
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub(super) fn meaning(&self) -> &str {
        &self.meaning
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSemanticDescription {
    revision: ApplicationProgramRevision,
    facts: Box<[ApplicationSemanticFact]>,
}

impl ApplicationSemanticDescription {
    pub fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }

    pub fn facts(&self) -> &[ApplicationSemanticFact] {
        &self.facts
    }

    pub(in crate::application_program) fn from_validated_parts(
        revision: ApplicationProgramRevision,
        features: &[ApplicationFeatureDeclaration],
        actions: &[ApplicationActionDeclaration],
        connections: &[ApplicationConnectionDeclaration],
        rules: &[ApplicationProgramRuleDeclaration],
    ) -> Self {
        let mut facts = Vec::new();
        append_features(&mut facts, features);
        append_actions(&mut facts, actions);
        append_connections(&mut facts, connections);
        append_rules(&mut facts, rules);
        facts.sort_unstable();
        Self {
            revision,
            facts: facts.into_boxed_slice(),
        }
    }
}

fn fact(
    family: ApplicationSemanticFamily,
    subject: impl Into<String>,
    meaning: impl Into<String>,
) -> ApplicationSemanticFact {
    ApplicationSemanticFact {
        family,
        subject: subject.into(),
        meaning: meaning.into(),
    }
}

fn append_features(
    facts: &mut Vec<ApplicationSemanticFact>,
    features: &[ApplicationFeatureDeclaration],
) {
    for feature in features {
        let owner = feature.composition_instance();
        let feature_subject = format!("{owner}|{}", feature.identity());
        facts.push(fact(
            ApplicationSemanticFamily::Features,
            &feature_subject,
            format!("{}.{}", feature.major(), feature.minor()),
        ));
        for input in feature.inputs() {
            facts.push(fact(
                ApplicationSemanticFamily::Ports,
                format!("{feature_subject}|input|{}", input.identity()),
                format!("required={}", input.required()),
            ));
        }
        for output in feature.outputs() {
            facts.push(fact(
                ApplicationSemanticFamily::Ports,
                format!("{feature_subject}|output|{}", output.identity()),
                "output",
            ));
        }
        append_feature_outputs(facts, &feature_subject, feature);
    }
}

fn append_feature_outputs(
    facts: &mut Vec<ApplicationSemanticFact>,
    feature_subject: &str,
    feature: &ApplicationFeatureDeclaration,
) {
    for artifact in feature.derived_artifacts() {
        let subject = format!("{feature_subject}|artifact|{}", artifact.identity());
        let mut dependencies = artifact
            .dependencies()
            .iter()
            .map(|dependency| dependency.identity())
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            &subject,
            format!(
                "output={}|locality={}:{}|retention={}|succession={}|required={}|producer={}|dependencies={}|reuse={}|stopped={}",
                artifact.output(), artifact.locality().identity(),
                artifact.locality().granule().canonical_token(), artifact.retention().canonical_token(),
                artifact.succession().canonical_token(), artifact.required(), artifact.producer_family(),
                dependencies.join(","), artifact.reuse_rule(), artifact.stopped_outcome()
            ),
        ));
        let resources = artifact.resource_ceiling();
        facts.push(fact(
            ApplicationSemanticFamily::Resources,
            subject,
            format!(
                "work={}|bytes={}",
                resources.maximum_work(),
                resources.maximum_retained_bytes()
            ),
        ));
    }
    for collection in feature.derived_collections() {
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            format!("{feature_subject}|collection|{}", collection.identity()),
            format!(
                "contributor={}|grouping={}|measures={}|lineage={}|applicability={}|incomplete={}|incremental={}",
                collection.contributor(), collection.grouping(), collection.measures(), collection.lineage(),
                collection.applicability(), collection.incomplete().canonical_token(), collection.incremental_update()
            ),
        ));
    }
    for computation in feature.managed_computations() {
        let subject = format!("{feature_subject}|computation|{}", computation.identity());
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            &subject,
            format!(
                "input={}|output={}|partition={}|reuse={}|stopped={}|execution={}|ordering={}",
                computation.input(),
                computation.output_artifact(),
                computation.partition(),
                computation.reuse(),
                computation.stopped(),
                computation.execution().canonical_token(),
                computation.ordering()
            ),
        ));
        let resources = computation.resources();
        facts.push(fact(
            ApplicationSemanticFamily::Resources,
            subject,
            format!(
                "work={}|bytes={}",
                resources.maximum_work(),
                resources.maximum_retained_bytes()
            ),
        ));
    }
}

fn append_actions(
    facts: &mut Vec<ApplicationSemanticFact>,
    actions: &[ApplicationActionDeclaration],
) {
    for action in actions {
        let subject = format!(
            "{}|{}|{}",
            action.composition_instance(),
            action.feature(),
            action.binding()
        );
        facts.push(fact(
            ApplicationSemanticFamily::Operations,
            &subject,
            format!(
                "input={}|conditional={}|required-output={}|invariant={}|observation={}|locality={}|granule={}|change={}|posture={}",
                action.operation_input_identity().as_str(), action.conditional_only(), action.required_output_source(),
                action.evaluated_requirement().map_or("", |value| value.identity()),
                action.correspondence().map_or("", |value| value.identity()),
                action.locality().map_or("", |value| value.identity()),
                action.locality().map_or("", |value| value.granule().canonical_token()),
                action.change_shape().map_or("", |value| value.identity()),
                action
                    .change_shape()
                    .map_or("", |value| value.posture().canonical_token()),
            ),
        ));
        if let Some(effect) = action.external_input() {
            facts.push(fact(
                ApplicationSemanticFamily::ExternalInputs,
                subject,
                effect.identity(),
            ));
        }
    }
}

fn append_connections(
    facts: &mut Vec<ApplicationSemanticFact>,
    connections: &[ApplicationConnectionDeclaration],
) {
    for connection in connections {
        facts.push(fact(
            ApplicationSemanticFamily::Connections,
            connection.identity(),
            format!(
                "{}:{}:{}|{}:{}:{}|required={}|exported={}",
                connection.source_instance(),
                connection.source_feature(),
                connection.source_port(),
                connection.target_instance(),
                connection.target_feature(),
                connection.target_port(),
                connection.target_required(),
                connection.exports_across_instances()
            ),
        ));
    }
}

fn append_rules(
    facts: &mut Vec<ApplicationSemanticFact>,
    rules: &[ApplicationProgramRuleDeclaration],
) {
    for rule in rules {
        facts.push(fact(
            ApplicationSemanticFamily::Rules,
            format!("{}|{}", rule.composition_instance(), rule.identity()),
            format!(
                "version={}.{}|point={}|owner={}",
                rule.major(),
                rule.minor(),
                rule.execution_point().canonical_token(),
                rule.local_owner().unwrap_or("")
            ),
        ));
    }
}
