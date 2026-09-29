#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIntentPayloadStop {
    ApplicationGenerationChanged,
    PublicationTransitionInFlight,
    NoCurrentPublication,
    Targeting(crate::runtime::interaction::UiInteractionTargetingDenial),
    ProjectionUnavailable {
        field: &'static str,
        projection: worth_ui_query_binding::WorthUiQueryViewIdentity,
    },
    ProjectionIdentityMismatch {
        field: &'static str,
        expected: worth_ui_query_binding::WorthUiQueryViewIdentity,
        observed: worth_ui_query_binding::WorthUiQueryViewIdentity,
    },
    ProjectionNotCurrent {
        field: &'static str,
        posture: worth_ui_query_binding::UiProjectionInputPosture,
    },
    ProjectionShapeMismatch {
        field: &'static str,
    },
    ProjectionValueMissing {
        field: &'static str,
    },
    TextByteBudgetExceeded {
        field: &'static str,
        observed: usize,
        maximum: usize,
    },
    DraftInteractionRequired {
        field: &'static str,
    },
    DraftFieldMismatch {
        field: &'static str,
    },
    SelectionInteractionRequired {
        field: &'static str,
    },
    SelectionProjectionMismatch {
        field: &'static str,
    },
    SelectionRevisionChanged {
        field: &'static str,
    },
    ApplicationFactUnavailable {
        field: &'static str,
        fact: Box<str>,
    },
    ApplicationFactIdentityChanged {
        field: &'static str,
        expected: Box<str>,
        observed: Box<str>,
    },
    ApplicationFactGenerationChanged {
        field: &'static str,
        fact: Box<str>,
    },
    ApplicationFactKindMismatch {
        field: &'static str,
        fact: Box<str>,
        observed: crate::capability::UiIntentPayloadFieldKind,
    },
    PayloadProjection(crate::capability::UiIntentPayloadProjectionViolation),
    /// The expression a payload field reads holds no current result of the
    /// type the field carries. The posture is the expression owner's own:
    /// never a default value and never `false`.
    ExpressionWithheld {
        field: &'static str,
        expression: Box<str>,
        posture: crate::runtime::expression::UiExpressionWithholding,
    },
    /// A `derived integer` holds a current value an unsigned 64-bit field
    /// cannot carry: it is refused, never clamped or truncated.
    DerivedIntegerOutOfRange {
        field: &'static str,
        expression: Box<str>,
        observed: i128,
    },
}
