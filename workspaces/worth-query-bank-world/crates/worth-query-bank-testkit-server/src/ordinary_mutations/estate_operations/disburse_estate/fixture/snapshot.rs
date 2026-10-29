pub use bank_domain::{
    accounting::{BankJournalEntry, BankPosting},
    model::{AccountName, BankSnapshotVersion, JournalEntryId, PostingId, SignedMoney},
    proposals::{BankSnapshot, BankSnapshotBuilder},
    schema::{AccountStatus, PostingPurpose},
};

pub use super::{
    ACTOR, BENEFICIARY, CASH, DECEASED, DESTINATION, EXECUTOR, INSTITUTION, SECOND_EXECUTOR, SOURCE,
};

pub const FUNDING_JOURNAL: JournalEntryId = JournalEntryId::new(15).unwrap();
pub const CASH_POSTING: PostingId = PostingId::new(16).unwrap();
pub const SOURCE_POSTING: PostingId = PostingId::new(17).unwrap();

pub fn snapshot(
    source_balance: i64,
    source_status: AccountStatus,
    destination_status: AccountStatus,
) -> BankSnapshot {
    BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(ACTOR)
        .principal(BENEFICIARY)
        .principal(EXECUTOR)
        .principal(SECOND_EXECUTOR)
        .institution_cash_account(CASH, INSTITUTION)
        .personal_account(
            SOURCE,
            INSTITUTION,
            DECEASED,
            AccountName::new("Estate source").unwrap(),
            source_status,
        )
        .personal_account(
            DESTINATION,
            INSTITUTION,
            BENEFICIARY,
            AccountName::new("Beneficiary destination").unwrap(),
            destination_status,
        )
        .projected_journal(funding_journal(source_balance))
        .build()
        .expect("the journal-funded disbursement snapshot should be valid")
}

pub fn funding_journal(source_balance: i64) -> BankJournalEntry {
    BankJournalEntry::from_projection(
        FUNDING_JOURNAL,
        PostingPurpose::OpeningFunding,
        vec![
            BankPosting::from_projection(
                CASH_POSTING,
                CASH,
                SignedMoney::from_minor(-source_balance),
            ),
            BankPosting::from_projection(
                SOURCE_POSTING,
                SOURCE,
                SignedMoney::from_minor(source_balance),
            ),
        ],
        None,
    )
}
