//! Domain algorithms for installed application invariants.
//!
//! Pure verdict, error, refusal, and access-denial types come from
//! `worth-query-execution-lineage`. Typed views, scope planning, and
//! `WorthQueryApplicationInvariantRule` live here.

use std::panic::{RefUnwindSafe, UnwindSafe};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

pub use worth_query_execution_lineage::facade::primary_graph::application_invariant::{
    access_denial, hook_error, refusal, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantVerdict, WorthQueryEmptyInvariantRefusal,
    WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind,
    WorthQueryInvariantRefusalDecodeDenial,
};
pub use worth_relational::facade::runtime::CustomInvariantInterruption;

pub mod content_fields;
mod prepared_scope;
mod program_selection;
mod read_observation;
mod registration;
mod typed_binding;
mod typed_view;

pub use super::aggregate_identity::WorthQueryAggregateIdentity;
pub use program_selection::WorthQueryProgramRuleSelection;
pub use registration::ErasedApplicationInvariantRule;
pub use typed_binding::{
    WorthQueryApplicationInvariantComponentSetBinding,
    WorthQueryApplicationInvariantDerivedRelationBinding,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantProjectedCaseBinding,
    WorthQueryApplicationInvariantRelationBinding,
};
pub use typed_view::{
    WorthQueryApplicationInvariantComponent, WorthQueryApplicationInvariantComponents,
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantDerivedMemberRelation,
    WorthQueryApplicationInvariantDerivedOrbit, WorthQueryApplicationInvariantDerivedRelation,
    WorthQueryApplicationInvariantDerivedWalk, WorthQueryApplicationInvariantDerivedWalkStep,
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantGraphComponent,
    WorthQueryApplicationInvariantGraphComponents, WorthQueryApplicationInvariantGraphMember,
    WorthQueryApplicationInvariantManagedIndexBinding, WorthQueryApplicationInvariantOrbit,
    WorthQueryApplicationInvariantReadView, WorthQueryApplicationInvariantRelation,
    WorthQueryApplicationInvariantScopePlanner, WorthQueryApplicationInvariantWalk,
    WorthQueryApplicationInvariantWalkIter, WorthQueryApplicationInvariantWalkStep,
    WorthQueryInvariantDerivedChanges, WorthQueryInvariantManagedIndexPair,
    WorthQueryInvariantManagedIndexRead, WorthQueryPreparedInvariantDerivedChanges,
    WorthQueryPreparedInvariantManagedIndex,
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
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError<Self::Refusal>>;

    fn supports_checked_preparation(&self) -> bool {
        false
    }

    fn prepare_scope_checked(
        &self,
        _planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        _budget: &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError<Self::Refusal>>> {
        None
    }

    fn evaluate_checked(
        &self,
        _context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        _scope: &Self::Scope,
        _budget: &dyn worth_relational::facade::runtime::CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Self::Refusal>,
            WorthQueryApplicationInvariantError<Self::Refusal>,
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
        WorthQueryApplicationInvariantError<Self::Refusal>,
    >;
}
