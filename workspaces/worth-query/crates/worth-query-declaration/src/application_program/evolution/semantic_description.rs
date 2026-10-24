mod connections;
mod feature_outputs;
mod feature_roles;
use super::super::semantic_encoding::{framed_fields, framed_parts};
use super::super::{
    ApplicationActionDeclaration, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramRevision, ApplicationProgramRuleDeclaration,
};
use connections::append_connections;
use feature_outputs::append_feature_outputs;

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
    Authorization,
}

impl ApplicationSemanticFamily {
    pub const ALL: [Self; 9] = [
        Self::Features,
        Self::Ports,
        Self::Connections,
        Self::Rules,
        Self::Operations,
        Self::ExternalInputs,
        Self::Outputs,
        Self::Resources,
        Self::Authorization,
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

const DELIVERY_FACET: &str = "delivery:";
const BODY_FACET: &str = "/body";

/// The action whose meaning an Operations fact subject describes.
///
/// An action owns its own fact plus its delivery and body facets. Action
/// subjects are length-framed, so they start with a digit and end exactly at
/// a frame boundary; neither facet marker can be mistaken for subject text.
pub fn operation_semantic_owner(subject: &str) -> &str {
    let subject = subject.strip_prefix(DELIVERY_FACET).unwrap_or(subject);
    match subject.strip_suffix(BODY_FACET) {
        Some(owner) if framed_whole(owner) => owner,
        _ => subject,
    }
}

fn framed_whole(mut encoded: &str) -> bool {
    while !encoded.is_empty() {
        let Some((length, rest)) = encoded.split_once(':') else {
            return false;
        };
        let Ok(length) = length.parse::<usize>() else {
            return false;
        };
        let Some(rest) = rest.get(length..) else {
            return false;
        };
        encoded = rest;
    }
    true
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
        adoption_authorization: Option<
            &super::super::ApplicationProgramAdoptionAuthorizationDeclaration,
        >,
    ) -> Self {
        let mut facts = Vec::new();
        append_features(&mut facts, features);
        append_actions(&mut facts, actions);
        append_connections(&mut facts, connections);
        append_rules(&mut facts, rules);
        if let Some(authorization) = adoption_authorization {
            facts.push(fact(
                ApplicationSemanticFamily::Authorization,
                "program-adoption-authorization",
                authorization.canonical_meaning(),
            ));
        }
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
        feature_roles::append_required_roles(facts, feature);
    }
}

fn append_actions(
    facts: &mut Vec<ApplicationSemanticFact>,
    actions: &[ApplicationActionDeclaration],
) {
    for action in actions {
        let subject = action.semantic_subject();
        if let Some(delivery) = action.delivery() {
            facts.push(fact(
                ApplicationSemanticFamily::Operations,
                format!("{DELIVERY_FACET}{subject}"),
                delivery.canonical_meaning(),
            ));
        }

        let mut fields = vec![
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
        ];
        if let Some(binding) = action.conditional_mutation_identity() {
            fields.push(("conditional-mutation-binding", binding.to_owned()));
        }
        {
            let requirements = action.workflow_execution_requirements();
            fields.push((
                "workflow-execution-requirements",
                requirements.persistence_identity(),
            ));
        }
        facts.push(fact(
            ApplicationSemanticFamily::Operations,
            &subject,
            framed_fields(fields),
        ));
        if let Some(body) = action.operation_body() {
            facts.push(fact(
                ApplicationSemanticFamily::Operations,
                format!("{subject}{BODY_FACET}"),
                crate::application_program::operation_body_encoding::operation_body_record(body),
            ));
        }
        if let Some(effect) = action.external_input() {
            facts.push(fact(
                ApplicationSemanticFamily::ExternalInputs,
                subject,
                framed_fields([("identity", effect.identity().to_owned())]),
            ));
        }
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
