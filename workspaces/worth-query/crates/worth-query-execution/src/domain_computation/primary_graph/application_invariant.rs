//! Domain algorithms for installed application invariants.
//!
//! Installation derives execution identity, applicability, access, and work limits
//! from the schema. Rules receive only the owner's bounded candidate views.

use std::panic::{RefUnwindSafe, UnwindSafe};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

pub use worth_relational::facade::runtime::{
    CustomInvariantExecutionError as WorthQueryApplicationInvariantExecutionError,
    CustomInvariantInterruption,
    CustomInvariantPreparationError as WorthQueryApplicationInvariantPreparationError,
};

pub(crate) mod refusal;
pub use refusal::{WorthQueryEmptyInvariantRefusal, WorthQueryInvariantRefusalDecodeDenial};

/// A domain violation is distinct from failure to evaluate and interruption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationInvariantVerdict<Refusal> {
    Pass,
    Violation(Refusal),
}

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
    type Refusal: 'static;
    /// The declared canonical application value codec, with a finite wire bound.
    /// Cause-free rules explicitly use `()` and `WorthQueryEmptyInvariantRefusal`.
    type RefusalBinding: worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding<Value = Self::Refusal>;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>;

    /// Declare `true` only when both checked hooks below are implemented.
    /// They must use the supplied caller budget for their bounded work and memory.
    /// A declaration does not authorize unchecked work, a replacement request,
    /// or a fallback to the ordinary hooks when that budget refuses.
    fn supports_checked_preparation(&self) -> bool {
        false
    }

    /// Prepare under the supplied budget, returning `Some` for a checked rule.
    /// Checkpoint on entry and before each unit of work; claim scratch or result
    /// bytes before growing or retaining your own storage. Stop immediately when
    /// any claim or checkpoint refuses. The owner retains the typed cause;
    /// neither `None`, ordinary preparation nor a new budget is a fallback.
    fn prepare_scope_checked(
        &self,
        _planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        _budget: &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        None
    }

    /// Evaluate under the same caller budget, returning `Some` for a checked rule.
    /// Checkpoint on entry and before each candidate or computation; claim memory
    /// before growing your own retained storage. A refusal ends evaluation: do
    /// not continue, report success, call the ordinary hook or mint a request.
    /// Return an error to exit; the owner's recorded typed stop takes precedence.
    fn evaluate_checked(
        &self,
        _context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        _scope: &Self::Scope,
        _budget: &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Self::Refusal>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        None
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Self::Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    >;
}
