use crate::capability::{UiIntentSchema, UiSemanticInteractionFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentInteractionPayloadSourceKind {
    CommittedDraft,
    ProjectionSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIntentCatalogPreparationDenial {
    DuplicateDeclaration {
        identity: Box<str>,
    },
    TooManyDeclarations {
        observed: usize,
        maximum: usize,
    },
    UnknownDefinition {
        declaration: Box<str>,
        definition: Box<str>,
    },
    PayloadSchemaMismatch {
        declaration: Box<str>,
        expected_identity: Box<str>,
        expected_version: u16,
        registered: UiIntentSchema,
    },
    OutcomeSchemaMismatch {
        declaration: Box<str>,
        expected_identity: Box<str>,
        expected_version: u16,
        registered: UiIntentSchema,
    },
    ConsequenceFamilyNotPermitted {
        declaration: Box<str>,
        family: crate::capability::UiIntentProductConsequenceFamilies,
    },
    UnknownConsequenceQuery {
        declaration: Box<str>,
        query: Box<str>,
    },
    InteractionNotAccepted {
        declaration: Box<str>,
        interaction: UiSemanticInteractionFamily,
    },
    DuplicatePayloadField {
        declaration: Box<str>,
        field: Box<str>,
    },
    MissingPayloadField {
        declaration: Box<str>,
        field: Box<str>,
    },
    UnknownPayloadField {
        declaration: Box<str>,
        field: Box<str>,
    },
    PayloadSourceKindMismatch {
        declaration: Box<str>,
        field: Box<str>,
        field_kind: crate::capability::UiIntentPayloadFieldKind,
        source_kind: crate::capability::UiIntentPayloadFieldKind,
    },
    PayloadConstantBudgetExceeded {
        declaration: Box<str>,
        field: Box<str>,
        observed: usize,
        maximum: usize,
    },
    InvalidPayloadProjectionIdentity {
        declaration: Box<str>,
        field: Box<str>,
        projection: Box<str>,
    },
    UnknownPayloadProjection {
        declaration: Box<str>,
        field: Box<str>,
        projection: Box<str>,
        required_shape: &'static str,
    },
    PayloadProjectionShapeMismatch {
        declaration: Box<str>,
        field: Box<str>,
        required_source: &'static str,
    },
    UnknownApplicationPayloadFact {
        declaration: Box<str>,
        field: Box<str>,
        fact: Box<str>,
    },
    ApplicationPayloadFactKindMismatch {
        declaration: Box<str>,
        field: Box<str>,
        fact: Box<str>,
        field_kind: crate::capability::UiIntentPayloadFieldKind,
        fact_kind: crate::capability::UiIntentPayloadFieldKind,
    },
    DuplicateInteractionPayloadSource {
        declaration: Box<str>,
        source: UiIntentInteractionPayloadSourceKind,
    },
    MissingInteractionPayloadSource {
        declaration: Box<str>,
        interaction: UiSemanticInteractionFamily,
        source: UiIntentInteractionPayloadSourceKind,
    },
    InteractionPayloadSourceMismatch {
        declaration: Box<str>,
        interaction: UiSemanticInteractionFamily,
        source: UiIntentInteractionPayloadSourceKind,
    },
    UnknownOperabilityApplicationFact {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        fact: Box<str>,
    },
    OperabilityApplicationFactKindMismatch {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        fact: Box<str>,
        observed: crate::capability::UiIntentPayloadFieldKind,
    },
    InvalidOperabilityProjectionIdentity {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        projection: Box<str>,
    },
    UnknownOperabilityProjection {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        projection: Box<str>,
    },
    OperabilityDraftSourceMismatch {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        interaction: UiSemanticInteractionFamily,
    },
    UnknownOperabilityCondition {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        condition: Box<str>,
    },
    OperabilityConditionRoleMismatch {
        declaration: Box<str>,
        axis: super::UiIntentOperabilityDependencyAxis,
        condition: Box<str>,
    },
    UnknownConfirmationApplicationFact {
        declaration: Box<str>,
        fact: Box<str>,
    },
    ConfirmationApplicationFactKindMismatch {
        declaration: Box<str>,
        fact: Box<str>,
        observed: crate::capability::UiIntentPayloadFieldKind,
    },
    UnknownRouteDeclaration {
        declaration: Box<str>,
    },
    MissingRouteTarget {
        authored_provenance_digest: u64,
    },
    ProductInteractionMismatch {
        declaration: Box<str>,
        declared: UiSemanticInteractionFamily,
        routed: UiSemanticInteractionFamily,
    },
    ConfirmationRequiresActivate {
        declaration: Box<str>,
        routed: UiSemanticInteractionFamily,
    },
    AmbiguousProductRoute {
        graph_node: crate::graph::UiGraphNodeIdentity,
        interaction: UiSemanticInteractionFamily,
    },
    AmbiguousConfirmationRoute {
        graph_node: crate::graph::UiGraphNodeIdentity,
        interaction: UiSemanticInteractionFamily,
    },
    RouteKindCrossover {
        graph_node: crate::graph::UiGraphNodeIdentity,
        interaction: UiSemanticInteractionFamily,
    },
    RouteCapacityExceeded {
        observed: usize,
        maximum: usize,
    },
}
