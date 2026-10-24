use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};

use super::BankSchema;

/// Every touched journal owns at least two balanced, purpose-consistent postings,
/// each attached to exactly one account.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPostingIntegrity;

impl ApplicationInvariantMarkerIdentity<BankSchema> for BankPostingIntegrity {
    const IDENTIFIER: &'static str = "BankPostingIntegrity";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(crate) fn posting_integrity_invariant(
) -> ApplicationInvariantDefinition<BankSchema, BankPostingIntegrity> {
    ApplicationInvariantDefinition::new(
        BankPostingIntegrity::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        // No declared work budget: a commit's evaluation grows with the
        // journals it touches, and its request's own policy bounds it.
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::RelationIntegrity],
            [
                ApplicationInvariantScopeTarget::Entity("JournalEntry".to_owned()),
                ApplicationInvariantScopeTarget::Entity("Posting".to_owned()),
                ApplicationInvariantScopeTarget::Entity("Account".to_owned()),
                ApplicationInvariantScopeTarget::Relation("JournalPosting".to_owned()),
                ApplicationInvariantScopeTarget::Relation("PostingAccount".to_owned()),
            ],
            [
                ApplicationInvariantScopeTarget::Entity("JournalEntry".to_owned()),
                ApplicationInvariantScopeTarget::Entity("Posting".to_owned()),
                ApplicationInvariantScopeTarget::Relation("JournalPosting".to_owned()),
                ApplicationInvariantScopeTarget::Relation("PostingAccount".to_owned()),
            ],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
