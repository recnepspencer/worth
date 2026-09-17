use bank_domain::schema::{
    BankSchema, JournalEntry, JournalPosting, JournalPurpose, Posting, PostingAccount,
    PostingAmount, Purpose,
};
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantExecutionError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantPreparationError,
    WorthQueryApplicationInvariantRelationBinding, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict,
};

pub(super) fn resolve_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, BankSchema>,
) -> Result<BankPostingIntegrityRule, String> {
    Ok(BankPostingIntegrityRule {
        journal: resolver
            .typed_entity(JournalEntry::reference())
            .ok_or("missing bank journal entity")?,
        journal_purpose: resolver
            .typed_field(JournalPurpose::reference())
            .ok_or("missing bank journal purpose field")?,
        journal_posting: resolver
            .typed_relation(JournalPosting::reference())
            .ok_or("missing bank journal-posting relation")?,
        posting_amount: resolver
            .typed_field(PostingAmount::reference())
            .ok_or("missing bank posting amount field")?,
        posting_purpose: resolver
            .typed_field(Purpose::reference())
            .ok_or("missing bank posting purpose field")?,
        posting_account: resolver
            .typed_relation(PostingAccount::reference())
            .ok_or("missing bank posting-account relation")?,
    })
}

pub(super) struct BankPostingIntegrityRule {
    journal: WorthQueryApplicationInvariantEntityBinding<BankSchema, JournalEntry>,
    journal_purpose: WorthQueryApplicationInvariantFieldBinding<
        BankSchema,
        JournalEntry,
        bank_domain::schema::PostingPurpose,
    >,
    journal_posting: WorthQueryApplicationInvariantRelationBinding<
        BankSchema,
        JournalPosting,
        JournalEntry,
        Posting,
    >,
    posting_amount: WorthQueryApplicationInvariantFieldBinding<
        BankSchema,
        Posting,
        bank_domain::model::SignedMoney<bank_domain::model::USD>,
    >,
    posting_purpose: WorthQueryApplicationInvariantFieldBinding<
        BankSchema,
        Posting,
        bank_domain::schema::PostingPurpose,
    >,
    posting_account: WorthQueryApplicationInvariantRelationBinding<
        BankSchema,
        PostingAccount,
        Posting,
        bank_domain::schema::Account,
    >,
}

impl WorthQueryApplicationInvariantRule<BankSchema> for BankPostingIntegrityRule {
    type Scope = Vec<WorthQueryApplicationInvariantEntity<BankSchema, JournalEntry>>;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, BankSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        let proposed = planner.proposed();
        let journals = proposed
            .touched_entities_of(&self.journal)
            .map_err(preparation_error)?;
        for journal in &journals {
            proposed
                .field(&self.journal_purpose, journal)
                .map_err(preparation_error)?;
            for relation in proposed
                .relations_from(&self.journal_posting, journal)
                .map_err(preparation_error)?
            {
                let posting = relation.to();
                proposed
                    .field(&self.posting_amount, posting)
                    .map_err(preparation_error)?;
                proposed
                    .field(&self.posting_purpose, posting)
                    .map_err(preparation_error)?;
                proposed
                    .relations_from(&self.posting_account, posting)
                    .map_err(preparation_error)?;
            }
        }
        Ok(journals)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, BankSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for journal in scope {
            if !self.journal_is_balanced(context, journal)? {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

impl BankPostingIntegrityRule {
    fn journal_is_balanced(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, BankSchema>,
        journal: &WorthQueryApplicationInvariantEntity<BankSchema, JournalEntry>,
    ) -> Result<bool, WorthQueryApplicationInvariantExecutionError> {
        let proposed = context.proposed();
        let Some(journal_purpose) = proposed
            .field(&self.journal_purpose, journal)
            .map_err(execution_error)?
        else {
            return Ok(false);
        };
        let postings = proposed
            .relations_from(&self.journal_posting, journal)
            .map_err(execution_error)?;
        let mut balance = BankPostingBalance::default();
        for relation in postings {
            let posting = relation.to();
            let Some(amount) = proposed
                .field(&self.posting_amount, posting)
                .map_err(execution_error)?
            else {
                return Ok(false);
            };
            let Some(purpose) = proposed
                .field(&self.posting_purpose, posting)
                .map_err(execution_error)?
            else {
                return Ok(false);
            };
            let account_count = proposed
                .relations_from(&self.posting_account, posting)
                .map_err(execution_error)?
                .len();
            if !balance.add(
                journal_purpose,
                purpose,
                account_count,
                amount.minor_units(),
            ) {
                return Ok(false);
            }
        }
        Ok(balance.is_balanced())
    }
}

#[derive(Default)]
struct BankPostingBalance {
    count: usize,
    total: i64,
}

impl BankPostingBalance {
    fn add(
        &mut self,
        journal_purpose: bank_domain::schema::PostingPurpose,
        posting_purpose: bank_domain::schema::PostingPurpose,
        account_count: usize,
        amount: i64,
    ) -> bool {
        if posting_purpose != journal_purpose || account_count != 1 {
            return false;
        }
        let Some(total) = self.total.checked_add(amount) else {
            return false;
        };
        self.total = total;
        self.count += 1;
        true
    }

    fn is_balanced(&self) -> bool {
        self.count >= 2 && self.total == 0
    }
}

fn preparation_error(
    error: impl std::fmt::Display,
) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(error.to_string())
}

fn execution_error(error: impl std::fmt::Display) -> WorthQueryApplicationInvariantExecutionError {
    WorthQueryApplicationInvariantExecutionError::new(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::BankPostingBalance;
    use bank_domain::schema::PostingPurpose;

    #[test]
    fn bank_posting_balance_rejects_unbalanced_and_inconsistent_inputs() {
        let mut balanced = BankPostingBalance::default();
        assert!(balanced.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 1, -30));
        assert!(!balanced.is_balanced(), "one posting cannot form a journal");
        assert!(balanced.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 1, 30));
        assert!(balanced.is_balanced());

        let mut unbalanced = BankPostingBalance::default();
        assert!(unbalanced.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 1, -30));
        assert!(unbalanced.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 1, 29));
        assert!(!unbalanced.is_balanced());

        let mut inconsistent = BankPostingBalance::default();
        assert!(!inconsistent.add(PostingPurpose::Transfer, PostingPurpose::Deposit, 1, 30));
        assert!(!inconsistent.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 0, 30));
        assert!(!inconsistent.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 2, 30));
        assert!(!inconsistent.is_balanced());

        let mut overflow = BankPostingBalance::default();
        assert!(overflow.add(
            PostingPurpose::Transfer,
            PostingPurpose::Transfer,
            1,
            i64::MAX
        ));
        assert!(!overflow.add(PostingPurpose::Transfer, PostingPurpose::Transfer, 1, 1));
    }
}
