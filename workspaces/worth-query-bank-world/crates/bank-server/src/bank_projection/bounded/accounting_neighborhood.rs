use std::collections::BTreeSet;

use bank_domain::accounting::{BankJournalEntry, BankPosting};
use bank_domain::model::JournalEntryId;
use bank_domain::schema::*;
use worth_query_host::facade::domain::OperationReads;

use super::{BoundedProjectionState, JournalEntity, PostingEntity, ProjectionReader};
use crate::bank_projection::{missing_field, BankProjectionDenial};

impl BoundedProjectionState {
    pub(in crate::bank_projection) fn project_journal_neighborhood<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        id: JournalEntryId,
    ) -> Result<(), BankProjectionDenial>
    where
        InstitutionAccount: OperationReads<Operation>,

        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,

        PostingAccount: OperationReads<Operation>,
        JournalPosting: OperationReads<Operation>,

        JournalReversal: OperationReads<Operation>,
    {
        let journal =
            reader.resolve_entity(JournalIdentityField::at("JournalIdentityField"), id)?;
        let mut journals = BTreeSet::from([journal]);
        let incoming = self.projected_relations_to(
            reader,
            JournalReversal::reference(),
            journals
                .first()
                .expect("targeted journal set contains its resolved target"),
        )?;
        match incoming.as_slice() {
            [] => {}
            [reversal] => {
                journals.insert(reversal.from().clone());
            }
            _ => return Err(BankProjectionDenial::AmbiguousRelation("JournalReversal")),
        }
        self.project_journals(reader, journals)
    }

    fn project_journals<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        journals: BTreeSet<JournalEntity>,
    ) -> Result<(), BankProjectionDenial>
    where
        InstitutionAccount: OperationReads<Operation>,

        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,

        PostingAccount: OperationReads<Operation>,
        JournalPosting: OperationReads<Operation>,

        JournalReversal: OperationReads<Operation>,
    {
        let mut entries = Vec::with_capacity(journals.len());
        for journal in journals {
            let id = required(
                self.projected_field(
                    reader,
                    &journal,
                    JournalIdentityField::at("JournalIdentityField"),
                )?,
                "JournalIdentityField",
            )?;
            let canonical =
                reader.resolve_entity(JournalIdentityField::at("JournalIdentityField"), id)?;
            if canonical != journal {
                return Err(BankProjectionDenial::AmbiguousRelation(
                    "JournalIdentityField",
                ));
            }
            let purpose = required(
                self.projected_field(reader, &journal, JournalPurpose::at("JournalPurpose"))?,
                "JournalPurpose",
            )?;
            let relations =
                self.projected_relations_from(reader, JournalPosting::reference(), &journal)?;
            if relations.len() < 2 {
                return Err(BankProjectionDenial::MissingRelation("JournalPosting"));
            }
            let mut postings = Vec::with_capacity(relations.len());
            let mut posting_ids = BTreeSet::new();
            for relation in relations {
                let posting = self.project_posting(reader, &journal, relation.to(), purpose)?;
                if !posting_ids.insert(posting.id()) {
                    return Err(BankProjectionDenial::AmbiguousRelation(
                        "PostingIdentityField",
                    ));
                }
                postings.push(posting);
            }
            postings.sort_by_key(BankPosting::id);
            entries.push(BankJournalEntry::from_projection(
                id,
                purpose,
                postings,
                reversal_target(self, reader, &journal)?,
            ));
        }
        entries.sort_by_key(BankJournalEntry::id);
        for entry in entries {
            self.update_builder(|builder| builder.projected_journal(entry));
        }
        Ok(())
    }

    fn project_posting<Operation>(
        &mut self,
        reader: &mut ProjectionReader<'_, '_, Operation>,
        journal: &JournalEntity,
        posting: &PostingEntity,
        journal_purpose: PostingPurpose,
    ) -> Result<BankPosting, BankProjectionDenial>
    where
        InstitutionAccount: OperationReads<Operation>,

        PersonalOwner: OperationReads<Operation>,
        BusinessAccount: OperationReads<Operation>,

        PostingAccount: OperationReads<Operation>,
        JournalPosting: OperationReads<Operation>,
    {
        let owners = self.projected_relations_to(reader, JournalPosting::reference(), posting)?;
        if !matches!(owners.as_slice(), [owner] if owner.from() == journal) {
            return Err(BankProjectionDenial::AmbiguousRelation("JournalPosting"));
        }
        let id = required(
            self.projected_field(
                reader,
                posting,
                PostingIdentityField::at("PostingIdentityField"),
            )?,
            "PostingIdentityField",
        )?;
        let canonical =
            reader.resolve_entity(PostingIdentityField::at("PostingIdentityField"), id)?;
        if &canonical != posting {
            return Err(BankProjectionDenial::AmbiguousRelation(
                "PostingIdentityField",
            ));
        }
        let purpose = required(
            self.projected_field(reader, posting, Purpose::at("Purpose"))?,
            "Purpose",
        )?;
        if purpose != journal_purpose {
            return Err(BankProjectionDenial::InvalidDomainState(
                bank_domain::proposals::BankProposalDenial::SnapshotInvariantViolated,
            ));
        }
        let accounts =
            self.projected_relations_from(reader, PostingAccount::reference(), posting)?;
        let [account] = accounts.as_slice() else {
            return Err(if accounts.is_empty() {
                BankProjectionDenial::MissingRelation("PostingAccount")
            } else {
                BankProjectionDenial::AmbiguousRelation("PostingAccount")
            });
        };
        self.project_account(reader, account.to())?;
        Ok(BankPosting::from_projection(
            id,
            required(
                self.projected_field(reader, account.to(), AccountIdentity::at("AccountIdentity"))?,
                "AccountIdentity",
            )?,
            required(
                self.projected_field(reader, posting, PostingAmount::at("PostingAmount"))?,
                "PostingAmount",
            )?,
        ))
    }
}

fn reversal_target<Operation>(
    state: &BoundedProjectionState,
    reader: &mut ProjectionReader<'_, '_, Operation>,
    journal: &JournalEntity,
) -> Result<Option<bank_domain::model::JournalEntryId>, BankProjectionDenial>
where
    JournalReversal: OperationReads<Operation>,
{
    let reversals =
        state.projected_relations_from(reader, JournalReversal::reference(), journal)?;
    match reversals.as_slice() {
        [] => Ok(None),
        [reversal] => required(
            state.projected_field(
                reader,
                reversal.to(),
                JournalIdentityField::at("JournalIdentityField"),
            )?,
            "JournalIdentityField",
        )
        .map(Some),
        _ => Err(BankProjectionDenial::AmbiguousRelation("JournalReversal")),
    }
}

fn required<Value>(
    value: Option<Value>,
    field: &'static str,
) -> Result<Value, BankProjectionDenial> {
    missing_field(value, field)
}
