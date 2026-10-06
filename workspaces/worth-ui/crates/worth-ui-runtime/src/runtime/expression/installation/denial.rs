use crate::capability::UiIntentPayloadFieldKind;

/// Why a sealed expression could not be installed into the catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionCatalogPreparationDenial {
    /// An operand reads an application fact the application never registered.
    UnknownApplicationFact {
        expression: Box<str>,
        operand: Box<str>,
        fact: Box<str>,
    },
    /// An operand reads a registered application fact of a different kind.
    ApplicationFactKindMismatch {
        expression: Box<str>,
        operand: Box<str>,
        fact: Box<str>,
        expected: UiIntentPayloadFieldKind,
        observed: UiIntentPayloadFieldKind,
    },
    /// An operand names a query scalar that the package does not declare as a
    /// scalar projection.
    UnknownQueryScalar {
        expression: Box<str>,
        operand: Box<str>,
        projection: Box<str>,
    },
    /// The declared scalar has no input slot in the application's query
    /// binding plan.
    UnboundProjection {
        expression: Box<str>,
        operand: Box<str>,
        view: Box<str>,
    },
    /// The operand reads a `require boolean` query scalar. The Query binding
    /// path carries text scalars only until Phase 3g of milestone 3.17 adds
    /// boolean scalar bindings, so no value can be bound for it yet.
    UnsupportedQueryScalarType {
        expression: Box<str>,
        operand: Box<str>,
        projection: Box<str>,
    },
    /// An operand names a condition or derived declaration that is not in the
    /// package.
    UnknownExpressionOperand {
        expression: Box<str>,
        operand: Box<str>,
        identity: Box<str>,
    },
    /// Expressions read each other in a cycle, so no evaluation order exists.
    ExpressionCycle { expression: Box<str> },
    /// The package declares more expressions than one catalog installs.
    CapacityExceeded { observed: usize, maximum: usize },
}
