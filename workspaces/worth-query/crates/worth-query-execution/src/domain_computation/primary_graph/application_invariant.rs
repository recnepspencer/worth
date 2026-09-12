//! Domain algorithms for installed application invariants.
//!
//! Installation derives execution identity, applicability, access, and work limits
//! from the schema. Rules receive only the owner's bounded candidate views.

use std::panic::{RefUnwindSafe, UnwindSafe};

pub use worth_relational::facade::runtime::{
    CustomInvariantExecutionContext as WorthQueryApplicationInvariantContext,
    CustomInvariantExecutionError as WorthQueryApplicationInvariantExecutionError,
    CustomInvariantPreparationError as WorthQueryApplicationInvariantPreparationError,
    CustomInvariantScopePlanner as WorthQueryApplicationInvariantScopePlanner,
    CustomInvariantVerdict as WorthQueryApplicationInvariantVerdict,
};

mod registration;
pub(super) use registration::ErasedApplicationInvariantRule;

/// Supplies domain validation without supplying its own installation authority.
pub trait WorthQueryApplicationInvariantRule:
    Send + Sync + RefUnwindSafe + UnwindSafe + 'static
{
    type Scope: Send + Sync + 'static;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>;

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>;
}
