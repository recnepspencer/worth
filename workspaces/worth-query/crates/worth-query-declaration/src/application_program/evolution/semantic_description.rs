use super::super::semantic_encoding::{framed_fields, framed_parts};
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

    /// Canonical descriptive meaning used by evolution and archive codecs.
    /// It grants no installation or execution authority.
    pub fn canonical_meaning(&self) -> &str {
        &self.meaning
    }
}

impl ApplicationActionDeclaration {
    /// Canonical descriptive subject used by semantic evolution evidence.
    ///
    /// The length framing keeps independently-authored identity components
    /// distinct even when an identity itself contains punctuation used by
    /// human-readable diagnostics. This remains descriptive evidence, not an
    /// execution or installation authority token.
    pub fn semantic_subject(&self) -> String {
        framed_parts(&[self.composition_instance(), self.feature(), self.binding()])
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
        let feature_subject = framed_parts(&[owner, feature.identity()]);
        facts.push(fact(
            ApplicationSemanticFamily::Features,
            &feature_subject,
            framed_fields([
                ("major", feature.major().to_string()),
                ("minor", feature.minor().to_string()),
            ]),
        ));
        for input in feature.inputs() {
            facts.push(fact(
                ApplicationSemanticFamily::Ports,
                framed_parts(&[owner, feature.identity(), "input", input.identity()]),
                framed_fields([("required", input.required().to_string())]),
            ));
        }
        for output in feature.outputs() {
            facts.push(fact(
                ApplicationSemanticFamily::Ports,
                framed_parts(&[owner, feature.identity(), "output", output.identity()]),
                framed_fields([("posture", "output".to_owned())]),
            ));
        }
        append_feature_outputs(facts, feature);
    }
}

fn append_feature_outputs(
    facts: &mut Vec<ApplicationSemanticFact>,
    feature: &ApplicationFeatureDeclaration,
) {
    let owner = feature.composition_instance();
    let feature_identity = feature.identity();
    for artifact in feature.derived_artifacts() {
        let subject = framed_parts(&[owner, feature_identity, "artifact", artifact.identity()]);
        let mut dependencies = artifact
            .dependencies()
            .iter()
            .map(|dependency| dependency.identity())
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            &subject,
            framed_fields(
                [
                    ("output", artifact.output().to_owned()),
                    ("locality", artifact.locality().identity().to_owned()),
                    (
                        "granule",
                        artifact.locality().granule().canonical_token().to_owned(),
                    ),
                    (
                        "retention",
                        artifact.retention().canonical_token().to_owned(),
                    ),
                    (
                        "succession",
                        artifact.succession().canonical_token().to_owned(),
                    ),
                    ("required", artifact.required().to_string()),
                    ("producer", artifact.producer_family().to_owned()),
                ]
                .into_iter()
                .chain(
                    dependencies
                        .into_iter()
                        .map(|dependency| ("dependency", dependency.to_owned())),
                )
                .chain([
                    ("reuse", artifact.reuse_rule().to_owned()),
                    ("stopped", artifact.stopped_outcome().to_owned()),
                ]),
            ),
        ));
        let resources = artifact.resource_ceiling();
        facts.push(fact(
            ApplicationSemanticFamily::Resources,
            subject,
            framed_fields([
                ("work", resources.maximum_work().to_string()),
                ("bytes", resources.maximum_retained_bytes().to_string()),
            ]),
        ));
    }
    for collection in feature.derived_collections() {
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            framed_parts(&[owner, feature_identity, "collection", collection.identity()]),
            framed_fields([
                ("contributor", collection.contributor().to_owned()),
                ("grouping", collection.grouping().to_owned()),
                ("measures", collection.measures().to_owned()),
                ("lineage", collection.lineage().to_owned()),
                ("applicability", collection.applicability().to_owned()),
                (
                    "incomplete",
                    collection.incomplete().canonical_token().to_owned(),
                ),
                ("incremental", collection.incremental_update().to_owned()),
            ]),
        ));
    }
    for computation in feature.managed_computations() {
        let subject = framed_parts(&[
            owner,
            feature_identity,
            "computation",
            computation.identity(),
        ]);
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            &subject,
            framed_fields([
                ("input", computation.input().to_owned()),
                ("output", computation.output_artifact().to_owned()),
                ("partition", computation.partition().to_owned()),
                ("reuse", computation.reuse().to_owned()),
                ("stopped", computation.stopped().to_owned()),
                (
                    "execution",
                    computation.execution().canonical_token().to_owned(),
                ),
                ("ordering", computation.ordering().to_owned()),
            ]),
        ));
        let resources = computation.resources();
        facts.push(fact(
            ApplicationSemanticFamily::Resources,
            subject,
            framed_fields([
                ("work", resources.maximum_work().to_string()),
                ("bytes", resources.maximum_retained_bytes().to_string()),
            ]),
        ));
    }
}

fn append_actions(
    facts: &mut Vec<ApplicationSemanticFact>,
    actions: &[ApplicationActionDeclaration],
) {
    for action in actions {
        let subject = action.semantic_subject();
        facts.push(fact(
            ApplicationSemanticFamily::Operations,
            &subject,
            framed_fields([
                (
                    "input",
                    action.operation_input_identity().as_str().to_owned(),
                ),
                ("conditional", action.conditional_only().to_string()),
                (
                    "required-output",
                    action.required_output_source().to_string(),
                ),
                (
                    "invariant",
                    action
                        .evaluated_requirement()
                        .map_or("", |value| value.identity())
                        .to_owned(),
                ),
                (
                    "observation",
                    action
                        .correspondence()
                        .map_or("", |value| value.identity())
                        .to_owned(),
                ),
                (
                    "locality",
                    action
                        .locality()
                        .map_or("", |value| value.identity())
                        .to_owned(),
                ),
                (
                    "granule",
                    action
                        .locality()
                        .map_or("", |value| value.granule().canonical_token())
                        .to_owned(),
                ),
                (
                    "change",
                    action
                        .change_shape()
                        .map_or("", |value| value.identity())
                        .to_owned(),
                ),
                (
                    "posture",
                    action
                        .change_shape()
                        .map_or("", |value| value.posture().canonical_token())
                        .to_owned(),
                ),
            ]),
        ));
        if let Some(effect) = action.external_input() {
            facts.push(fact(
                ApplicationSemanticFamily::ExternalInputs,
                subject,
                framed_fields([("identity", effect.identity().to_owned())]),
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
            framed_parts(&[
                connection.source_instance(),
                connection.target_instance(),
                connection.identity(),
            ]),
            framed_fields([
                ("source-instance", connection.source_instance().to_owned()),
                ("source-feature", connection.source_feature().to_owned()),
                ("source-port", connection.source_port().to_owned()),
                ("target-instance", connection.target_instance().to_owned()),
                ("target-feature", connection.target_feature().to_owned()),
                ("target-port", connection.target_port().to_owned()),
                ("required", connection.target_required().to_string()),
                (
                    "exported",
                    connection.exports_across_instances().to_string(),
                ),
            ]),
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
            framed_parts(&[rule.composition_instance(), rule.identity()]),
            framed_fields([
                ("major", rule.major().to_string()),
                ("minor", rule.minor().to_string()),
                ("point", rule.execution_point().canonical_token().to_owned()),
                ("owner", rule.local_owner().unwrap_or("").to_owned()),
            ]),
        ));
    }
}
