use bank_domain::{
    model::{AccountName, BankSnapshotVersion},
    proposals::{BankSnapshot, BankSnapshotBuilder},
    schema::AccountStatus,
};

use super::super::{
    extra_principal, ACCOUNT, ALTERNATE_INSTITUTION, APPROVER, DECEASED, EXECUTOR, INSTITUTION,
    OTHER_ACCOUNT, REVIEWER, SPECIALIST,
};

pub(super) fn snapshot(unrelated_grants: usize) -> BankSnapshot {
    snapshot_builder(unrelated_grants)
        .build()
        .expect("capability fixture snapshot should be valid")
}

pub(super) fn live_snapshot() -> BankSnapshot {
    snapshot_builder(0)
        .institution_cash_account(
            bank_domain::model::AccountId::new(100_000).unwrap(),
            INSTITUTION,
        )
        .build()
        .expect("the live fixture includes real institution cash")
}

fn snapshot_builder(unrelated_grants: usize) -> BankSnapshotBuilder {
    let mut snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .institution(ALTERNATE_INSTITUTION)
        .principal(DECEASED)
        .principal(SPECIALIST)
        .principal(EXECUTOR)
        .principal(APPROVER)
        .principal(REVIEWER)
        .personal_account(
            ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Estate Operating").unwrap(),
            AccountStatus::Frozen,
        )
        .personal_account(
            OTHER_ACCOUNT,
            INSTITUTION,
            EXECUTOR,
            AccountName::new("Executor Settlement").unwrap(),
            AccountStatus::Open,
        );
    for ordinal in 0..unrelated_grants {
        snapshot = snapshot.principal(extra_principal(ordinal));
    }
    snapshot
}
