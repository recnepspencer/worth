use std::marker::PhantomData;

use worth_foundational::facade::CanonicalDigestDerivationDenial;

use super::{ApplicationWorkflowSpec, AuthoredWorkflowDefinition, ValidatedWorkflowDefinition};

mod connections;
mod control;
mod dominance;
mod index;
mod limits;
mod work;

use work::ValidationWorkMeter;
pub use work::{
    ApplicationWorkflowRetryValidationComplexityContract,
    ApplicationWorkflowValidationComplexityContract, ApplicationWorkflowValidationWork,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowValidationDenialKind {
    MissingStart,
    UnknownStart,
    NodeLimitExceeded,
    ConnectionLimitExceeded,
    EffectLimitExceeded,
    ComponentOccurrenceLimitExceeded,
    ComponentDepthExceeded,
    NodeProvenanceLimitExceeded,
    ConnectionProvenanceLimitExceeded,
    PortProvenanceLimitExceeded,
    UnknownConnectionEndpoint,
    DuplicateConnection,
    InvalidDataFlow,
    IncompatibleOperationInput,
    UnavailableDataFlow,
    MissingAssessmentSubject,
    MissingConditionSubject,
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
    let mut validation_work = ValidationWorkMeter::default();
    limits::enforce(&authored, &mut validation_work)?;
    let graph = index::ValidationGraph::build(
        &authored.nodes,
        &authored.connections,
        &mut validation_work,
    )?;
    let Some(start_index) = graph.resolve(&start, &mut validation_work) else {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::UnknownStart,
            start.as_str(),
        ));
    };
    connections::validate(&graph, &mut validation_work)?;
    connections::validate_requirements(&graph, &mut validation_work)?;
    let control = control::validate(start_index, &graph, &mut validation_work)?;
    dominance::validate_availability(start_index, &graph, &control, &mut validation_work)?;
    let validation_work = validation_work.finish();
    drop(graph);
    let (content_identity, nodes, connections) = super::canonical::canonicalize::<Spec>(
        authored.limits,
        start.as_str(),
        authored.nodes,
        authored.connections,
    )
    .map_err(canonical_denial)?;
    let mut component_expansions = authored.component_expansions;
    component_expansions.sort_by(|left, right| {
        left.occurrence_path()
            .cmp(right.occurrence_path())
            .then_with(|| left.component().cmp(right.component()))
    });
    Ok(ValidatedWorkflowDefinition {
        identity: authored.identity,
        content_identity,
        limits: authored.limits,
        start,
        nodes,
        connections,
        component_expansions: component_expansions.into_boxed_slice(),
        validation_work,
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
