//! Domain algorithms for installed application invariants.
//!
//! Installation derives execution identity, applicability, access, and work limits
//! from the schema. Rules receive only the owner's bounded candidate views.

use std::panic::{RefUnwindSafe, UnwindSafe};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

pub use worth_relational::facade::runtime::{
    CustomInvariantExecutionError as WorthQueryApplicationInvariantExecutionError,
    CustomInvariantPreparationError as WorthQueryApplicationInvariantPreparationError,
    CustomInvariantVerdict as WorthQueryApplicationInvariantVerdict,
};

mod access_denial;
mod prepared_scope;
mod program_selection;
mod registration;
mod typed_binding;
mod typed_view;
pub use access_denial::{WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind};
pub(super) use program_selection::WorthQueryProgramRuleSelection;
pub(super) use registration::ErasedApplicationInvariantRule;
pub use typed_binding::{
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantRelationBinding,
};
pub use typed_view::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantReadView, WorthQueryApplicationInvariantRelation,
    WorthQueryApplicationInvariantScopePlanner,
};

/// Supplies domain validation without supplying its own installation authority.
pub trait WorthQueryApplicationInvariantRule<Schema>:
    Send + Sync + RefUnwindSafe + UnwindSafe + 'static
where
    Schema: ApplicationSchema,
{
    type Scope: Send + Sync + 'static;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>;

    /// Declares that this rule checkpoints preparation under a caller budget.
    fn supports_checked_preparation(&self) -> bool {
        false
    }

    /// Returns no checked implementation when the rule cannot honor the budget.
    fn prepare_scope_checked(
        &self,
        _planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        _budget: &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        None
    }

    /// Evaluates cooperatively; every candidate and retained growth uses the budget.
    fn evaluate_checked(
        &self,
        _context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        _scope: &Self::Scope,
        _budget: &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>,
    > {
        None
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>;
}
