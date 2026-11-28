//! The ordinary Home carries the Bank workflow vocabulary across graceful reopening.
use bank_domain::{
    model::{BankPrincipalId, BankSnapshotVersion},
    proposals::BankSnapshotBuilder,
    schema::ApprovedBusinessPaymentWorkflow,
};
use bank_server::{open_bank_application_home, BankPrincipalSeed, BankWorldSeed};
use worth_query_bank_testkit_server::facade::support::DynamicIdentity;
use worth_query_host::facade::{WorthQueryApplicationHomeLocation, WorthQueryHomeOpenDisposition};

#[test]
fn directory_home_keeps_bank_workflow_custody_through_close_and_reopen() {
    let directory = std::env::temp_dir().join(format!(
        "worth-bank-workflow-custody-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let identity = DynamicIdentity::new("directory-workflow-custody");
    let owner = BankPrincipalId::new(41).unwrap();

    let mut first_revision = None;
    for posture in [
        WorthQueryHomeOpenDisposition::Created,
        WorthQueryHomeOpenDisposition::Opened,
    ] {
        let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
            .principal(owner)
            .build()
            .unwrap();
        let seed = BankWorldSeed::new(snapshot)
            .principal(BankPrincipalSeed::enabled(owner, identity.external()));
        let bank = open_bank_application_home(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            seed,
        )
        .unwrap();
        let revision = *bank.approved_payment_workflow_spec().program_revision();
        assert_eq!(*first_revision.get_or_insert(revision), revision);

        let home = bank
            .into_home()
            .ok()
            .expect("Bank retained its Directory Home");
        assert_eq!(home.open_disposition(), posture);
        let vocabulary = home
            .workflow_vocabulary::<ApprovedBusinessPaymentWorkflow>()
            .unwrap();
        assert!(std::ptr::eq(vocabulary.runtime(), home.primary_graph()));
        home.close_clean().unwrap();
    }
    std::fs::remove_dir_all(directory).unwrap();
}
