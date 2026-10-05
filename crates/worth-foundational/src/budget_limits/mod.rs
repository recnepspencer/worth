//! A limit that ran out, minted only by the budget that owns it.
//!
//! A refusal for want of budget says nothing about the media or the input: the
//! same work may pass under a wider budget. Damage says the opposite. The two
//! must never be confused, and a limit must carry the counts the budget really
//! had, not numbers made up where the refusal was reported.
//!
//! [`ExhaustedLimit`] makes that a type law. Each budget owner declares:
//! - a dimension enum implementing [`LimitDimension`], naming what ran out;
//! - a sealed authority marker (`worth_proof::authority_marker!`) in a *leaf*
//!   module, because the declaring module's descendants can mint too;
//! - the budget whose refuse path alone records the refusal with
//!   [`Performed::record`](worth_proof::Performed::record) and calls
//!   [`ExhaustedLimit::refused`].
//!
//! Coherence gives each dimension exactly one `Authority`, so only that
//! owner's witness can produce the `Performed` the constructor demands:
//!
//! ```
//! mod entries_budget {
//!     use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};
//!     use worth_proof::Performed;
//!     worth_proof::authority_marker!(pub EntriesAuthority);
//!
//!     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//!     pub enum EntriesBound { Entries }
//!     impl LimitDimension for EntriesBound { type Authority = EntriesAuthority; }
//!
//!     pub struct EntriesBudget { admitted: u64 }
//!     impl EntriesBudget {
//!         pub fn new(admitted: u64) -> Self { Self { admitted } }
//!         /// The only door: a count past the budget is refused with both counts.
//!         pub fn admit(&self, count: u64) -> Result<(), ExhaustedLimit<EntriesBound>> {
//!             if count <= self.admitted { return Ok(()); }
//!             let counts = LimitCounts::new(count, self.admitted);
//!             let refusal = Performed::<BudgetRefused, _, _>::record(&EntriesAuthority::witness(), counts);
//!             Err(ExhaustedLimit::refused(EntriesBound::Entries, refusal))
//!         }
//!     }
//! }
//!
//! let limit = entries_budget::EntriesBudget::new(4).admit(5).unwrap_err();
//! assert_eq!(limit.dimension(), entries_budget::EntriesBound::Entries);
//! assert_eq!((limit.observed(), limit.admitted()), (5, 4));
//! ```
//!
//! What this does *not* check: a limit reported as damage still compiles,
//! because any `match` arm can return a damage variant. Keep damage variants
//! specific, with no catch-all, so a reviewer sees that lie.

mod exhausted_limit;

pub use exhausted_limit::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};

use crate::facade::ResponsibilityArea;

pub fn responsibility() -> ResponsibilityArea {
    ResponsibilityArea::new(
        "budget_limits",
        "the exhausted-limit vocabulary: limit dimensions tied to the authority of the budget that owns them, observed and admitted counts, and the refusal evidence that alone mints an exhausted limit",
        "any budget, its admitted values, or the damage vocabulary of an owning runtime",
    )
}
