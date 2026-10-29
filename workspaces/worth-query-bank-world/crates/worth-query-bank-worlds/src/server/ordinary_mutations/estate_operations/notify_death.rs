#[path = "notify_death/fixture.rs"]
pub mod fixture;

pub use bank_domain::{
    estate::DeathNoticeStatus, model::BankPrincipalId, proposals::BankIdempotencyKey,
};
pub use bank_server::{queries, BankDeathNotificationProjectionDenial, BankReadControls};

pub use self::fixture::{notification_world, NotificationFixture};
pub use crate::facade::server::support::request_scope;

pub fn notify(
    fixture: &NotificationFixture,
    notice: bank_domain::estate::DeathNoticeId,
    subject: BankPrincipalId,
    identity: u8,
) -> super::EstateSourceAdvance<
    bank_domain::schema::NotifyEstateDeath,
    bank_server::DeathNotificationAction,
> {
    let key = idempotency(identity);
    {
        let source_actor = &fixture.authenticate_specialist();
        let source_scope = &request_scope();
        let source_key = &key;
        let source_intent =
            bank_domain::schema::NotifyEstateDeath::try_from(fixture.action(notice, subject))
                .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::notify_estate_death(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        source_operation.advance(&source_request)
    }
}

pub fn notice_status(fixture: &NotificationFixture) -> DeathNoticeStatus {
    fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&fixture.authenticate_specialist())
        .controls(BankReadControls::current(request_scope(), 16).unwrap())
        .execute()
        .expect("the assigned specialist should observe the death notice")
        .rows()[0]
        .death_notice()
        .status()
}

pub fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("notify-death-{identity}")).unwrap()
}

pub fn assert_admission_derives_the_request_identities_once(
    phases: bank_server::BankCommitCanonicalWorkPhases,
) {
    // Binding-selected capability admission governs its input as well as the mutation
    // input/key identities. All three native derivations are included here; later
    // publication and domain recovery do not rebuild these identities.
    let admission = phases.admission();
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(admission.digest_derivations(), 3);
    assert_eq!(admission.canonical_entries(), 3);
    assert_eq!(admission.canonical_encoded_bytes(), 256);
    assert_eq!(admission.sha256_input_bytes(), 475);
    assert_eq!(admission.sha256_compression_blocks(), 9);
    assert_eq!(admission.digest_text_materializations(), 0);
    for work in [
        phases.installation(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.retry_resolution(),
        phases.publication(),
    ] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.canonical_encoded_bytes(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}
