use std::collections::BTreeMap;
use std::marker::PhantomData;

use worth_foundational::facade::CanonicalDigestDerivationDenial;

use super::{ApplicationWorkflowSpec, AuthoredWorkflowDefinition, ValidatedWorkflowDefinition};

mod connections;
mod control;
mod limits;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowValidationDenialKind {
    MissingStart,
    UnknownStart,
    NodeLimitExceeded,
    ConnectionLimitExceeded,
    EffectLimitExceeded,
    ComponentDepthExceeded,
    UnknownConnectionEndpoint,
    DuplicateConnection,
    InvalidDataFlow,
    UnavailableDataFlow,
    MissingAssessmentSubject,
    IncompleteEvidenceJoin,
    IncompleteApproval,
    MissingWorkflowAuthority,
    UnexpectedWorkflowAuthority,
    MissingControlOutcome,
    AmbiguousControlOutcome,
    UnexpectedControlOutcome,
    TerminalHasSuccessor,
    UnreachableNode,
    ControlCycle,
    MissingTerminal,
    CanonicalBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowValidationDenial {
    kind: ApplicationWorkflowValidationDenialKind,
    subject: String,
}

impl ApplicationWorkflowValidationDenial {
    pub const fn kind(&self) -> ApplicationWorkflowValidationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for ApplicationWorkflowValidationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow definition denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for ApplicationWorkflowValidationDenial {}

pub(super) fn validate<Spec>(
    authored: AuthoredWorkflowDefinition<Spec>,
) -> Result<ValidatedWorkflowDefinition<Spec>, ApplicationWorkflowValidationDenial>
where
    Spec: ApplicationWorkflowSpec,
{
    let start = authored.start.clone().ok_or_else(|| {
        denial(
            ApplicationWorkflowValidationDenialKind::MissingStart,
            authored.identity.as_str(),
        )
    })?;
    limits::enforce(&authored)?;
    let nodes = authored
        .nodes
        .iter()
        .map(|node| (node.identity(), node))
        .collect::<BTreeMap<_, _>>();
    if !nodes.contains_key(&start) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::UnknownStart,
            start.as_str(),
        ));
    }
    connections::validate(&nodes, &authored.connections)?;
    connections::validate_requirements(&nodes, &authored.connections)?;
    control::validate(&start, &nodes, &authored.connections)?;
    connections::validate_availability(&start, &nodes, &authored.connections)?;
    let (content_identity, nodes, connections) = super::canonical::canonicalize::<Spec>(
        authored.limits,
        start.as_str(),
        authored.nodes,
        authored.connections,
    )
    .map_err(canonical_denial)?;
    Ok(ValidatedWorkflowDefinition {
        identity: authored.identity,
        content_identity,
        limits: authored.limits,
        start,
        nodes,
        connections,
        marker: PhantomData,
    })
}

fn canonical_denial(
    denial_value: CanonicalDigestDerivationDenial,
) -> ApplicationWorkflowValidationDenial {
    denial(
        ApplicationWorkflowValidationDenialKind::CanonicalBudgetExceeded,
        format!("{denial_value:?}"),
    )
}

fn denial(
    kind: ApplicationWorkflowValidationDenialKind,
    subject: impl Into<String>,
) -> ApplicationWorkflowValidationDenial {
    ApplicationWorkflowValidationDenial {
        kind,
        subject: subject.into(),
    }
}
