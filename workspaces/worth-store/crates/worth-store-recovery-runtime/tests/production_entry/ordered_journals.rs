//! The home's ordered journals: Relational's canonical commits and its branch
//! transitions, each a contiguous sequence under its own anchor, published,
//! read, resolved and recovered through the production path.

use super::*;

#[path = "ordered_journals/anchor_table.rs"]
mod anchor_table;
#[path = "ordered_journals/branch_transitions_oracle.rs"]
mod branch_transitions_oracle;
#[path = "canonical_journal.rs"]
pub(crate) mod canonical_journal;
#[path = "canonical_journal_oracle.rs"]
pub(crate) mod canonical_journal_oracle;
#[path = "ordered_journals/two_journals.rs"]
pub(crate) mod two_journals;
#[path = "ordered_journals/unresolved_isolation.rs"]
mod unresolved_isolation;
