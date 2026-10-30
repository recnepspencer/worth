#[path = "freeze_account/correction.rs"]
pub mod correction;
#[path = "freeze_account/correction_owner.rs"]
pub mod correction_owner;
#[path = "freeze_account/fixture.rs"]
pub mod fixture;
#[path = "freeze_account/reapplication_owner.rs"]
pub mod reapplication_owner;

pub use bank_domain::{
    estate::EstateWorkflowStage, proposals::BankIdempotencyKey, schema::AccountStatus,
};
pub use bank_server::{queries, BankEstateFreezeProjectionDenial, BankReadControls};

pub use self::fixture::{exact_freeze_world, foreign_account_freeze_world, FreezeFixture};
pub use crate::facade::support::request_scope;

pub fn estate_account_status(fixture: &FreezeFixture) -> AccountStatus {
    let specialist = fixture.authenticate_specialist();
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(read_controls())
        .execute()
        .expect("the assigned specialist should observe the estate account");
    let overview = &result.rows()[0];
    assert_eq!(overview.stage(), EstateWorkflowStage::Administration);
    assert_eq!(overview.account().id(), fixture.estate_account);
    overview.account().status()
}

pub fn foreign_account_status(fixture: &FreezeFixture) -> AccountStatus {
    let owner = fixture.authenticate_foreign_owner();
    fixture
        .world
        .runtime
        .query(queries::account_summary(fixture.foreign_account))
        .as_principal(&owner)
        .controls(read_controls())
        .execute()
        .expect("the foreign account owner should observe their account")
        .rows()[0]
        .status()
}

pub fn read_controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 16).unwrap()
}

pub fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("freeze-estate-account-{identity}")).unwrap()
}

pub fn assert_freeze_canonical_work(phases: bank_server::BankCommitCanonicalWorkPhases) {
    // Binding-selected capability admission governs its input as well as the mutation
    // input/key identities. All three native derivations are included here; later
    // publication and domain recovery do not rebuild these identities.
    let admission = phases.admission();
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(admission.digest_derivations(), 3);
    assert_eq!(admission.canonical_entries(), 3);
    assert_eq!(admission.canonical_encoded_bytes(), 269);
    assert_eq!(admission.sha256_input_bytes(), 488);
    assert_eq!(admission.sha256_compression_blocks(), 9);
    assert_eq!(admission.digest_text_materializations(), 0);
    for work in [
        phases.installation(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.live_delivery(),
        phases.retry_resolution(),
        phases.recovery_inspection(),
        phases.publication(),
    ] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.canonical_encoded_bytes(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}
