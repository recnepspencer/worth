//! A typed declaration expression is resolved only by the matching operation builder.
use super::{ApplicationCorrectionBinding, DeclaredCorrectionBinding};
use crate::application_schema::{
    ApplicationFieldBindingRecipe, ApplicationSchema, ApplicationSchemaDeclarationDenial,
};
use std::any::TypeId;

#[derive(Clone, Debug)]
pub(crate) struct CorrectionBindingDeclaration {
    marker: TypeId,
    pub(crate) identity: &'static str,
    compose: fn(
        Option<super::super::DeclaredReconciliationProcedure>,
    ) -> Result<ComposedCorrectionBinding, ApplicationSchemaDeclarationDenial>,
}
impl CorrectionBindingDeclaration {
    pub(crate) fn of<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>() -> Self {
        Self {
            marker: TypeId::of::<C>(),
            identity: C::IDENTITY,
            compose: DeclaredCorrectionBinding::compose::<S, C>,
        }
    }
    pub(crate) fn compose(
        &self,
        reconciliation: Option<super::super::DeclaredReconciliationProcedure>,
    ) -> Result<ComposedCorrectionBinding, ApplicationSchemaDeclarationDenial> {
        (self.compose)(reconciliation)
    }
}
impl PartialEq for CorrectionBindingDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.marker == other.marker
    }
}
impl Eq for CorrectionBindingDeclaration {}
impl PartialOrd for CorrectionBindingDeclaration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CorrectionBindingDeclaration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.marker.cmp(&other.marker)
    }
}

/// Portable meaning and expectations come from the same authored expression.
pub(crate) struct ComposedCorrectionBinding {
    pub(crate) binding: DeclaredCorrectionBinding,
    pub(crate) fields: Vec<ApplicationFieldBindingRecipe>,
}
