//! Typed authoring of the same portable relation expression used by installation.
use std::marker::PhantomData;

use crate::application_schema::ApplicationRelationRef;

use super::expression::{
    ApplicationDerivedRelationExpression, ApplicationDerivedRelationSource,
    ApplicationDerivedRelationSourceKind,
};
use super::reference::ApplicationDerivedRelationRef;

mod sealed {
    pub trait Operand {}
}

/// A typed expression operand. Only declared relation references and recipes
/// implement it; resulting meaning still requires ordinary schema validation.
pub trait ApplicationDerivedRelationOperand<Schema, From, To>: sealed::Operand {
    /// Returns owned symbolic meaning; this does not acquire a graph source.
    fn into_expression(self) -> ApplicationDerivedRelationExpression;
}

/// A rebuildable endpoint-set recipe, distinct from a stored relation or permit.
pub struct ApplicationDerivedRelation<Schema, From, To> {
    pub(super) expression: ApplicationDerivedRelationExpression,
    marker: PhantomData<fn() -> (Schema, From, To)>,
}

impl<S, F, T> ApplicationDerivedRelation<S, F, T> {
    /// Composes endpoint-compatible legs in their declared direction.
    pub fn compose<M>(
        left: impl ApplicationDerivedRelationOperand<S, F, M>,
        right: impl ApplicationDerivedRelationOperand<S, M, T>,
    ) -> Self {
        Self {
            expression: left.into_expression().compose(right.into_expression()),
            marker: PhantomData,
        }
    }

    /// Appends another endpoint-compatible leg.
    pub fn then<End>(
        self,
        next: impl ApplicationDerivedRelationOperand<S, T, End>,
    ) -> ApplicationDerivedRelation<S, F, End> {
        ApplicationDerivedRelation {
            expression: self.expression.compose(next.into_expression()),
            marker: PhantomData,
        }
    }

    /// Reverses the complete endpoint-set recipe.
    pub fn reversed(self) -> ApplicationDerivedRelation<S, T, F> {
        ApplicationDerivedRelation {
            expression: self.expression.reverse(),
            marker: PhantomData,
        }
    }

    /// Starts a recipe from an actual stored or named-derived reference.
    pub fn relation(operand: impl ApplicationDerivedRelationOperand<S, F, T>) -> Self {
        Self {
            expression: operand.into_expression(),
            marker: PhantomData,
        }
    }
}

impl<S, E> ApplicationDerivedRelation<S, E, E> {
    /// Excludes equal endpoints from a homogeneous relation.
    pub fn without_self_edges(self) -> Self {
        Self {
            expression: self.expression.without_self_edges(),
            marker: PhantomData,
        }
    }
}

impl<S, F, T> sealed::Operand for ApplicationDerivedRelation<S, F, T> {}
impl<S, F, T> ApplicationDerivedRelationOperand<S, F, T> for ApplicationDerivedRelation<S, F, T> {
    fn into_expression(self) -> ApplicationDerivedRelationExpression {
        self.expression
    }
}

impl<S, R, F, T> sealed::Operand for ApplicationRelationRef<S, R, F, T> {}
impl<S, R, F, T> ApplicationDerivedRelationOperand<S, F, T> for ApplicationRelationRef<S, R, F, T> {
    fn into_expression(self) -> ApplicationDerivedRelationExpression {
        ApplicationDerivedRelationExpression::source(ApplicationDerivedRelationSource {
            kind: ApplicationDerivedRelationSourceKind::Stored,
            relation: self.name().to_owned(),
            from: self.from().to_owned(),
            to: self.to().to_owned(),
        })
    }
}

impl<S, R, F, T> sealed::Operand for ApplicationDerivedRelationRef<S, R, F, T> {}
impl<S, R, F, T> ApplicationDerivedRelationOperand<S, F, T>
    for ApplicationDerivedRelationRef<S, R, F, T>
{
    fn into_expression(self) -> ApplicationDerivedRelationExpression {
        ApplicationDerivedRelationExpression::source(ApplicationDerivedRelationSource {
            kind: ApplicationDerivedRelationSourceKind::Derived,
            relation: self.name().to_owned(),
            from: self.from().to_owned(),
            to: self.to().to_owned(),
        })
    }
}
