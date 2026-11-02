//! Retained v1 remains historical while current-only managed reads require selection.
use super::*;
#[test]
fn personal_creation_updates_warm_empty_source_and_preserves_v1_pinned_read() {
    let fixture = ordinary_read_world("managed-account-creation", 0);
    let stranger = fixture.authenticate(STRANGER);
    let teller = fixture.authenticate(TELLER);
    assert!(managed(&fixture, &stranger).is_empty());
    let scope = request_scope();
    let request = fixture.world.runtime.request(&stranger, &scope);
    let old = request.retain_current_read().unwrap();
    let before = work(&fixture.world.runtime);
    mutate!(
        fixture,
        &teller,
        mutations::create_personal_account(CreatePersonalAccount {
            institution: fixture.institution,
            owner: principal_id(STRANGER),
            display_name: AccountName::new("Managed new owner").unwrap(),
        }),
        "managed-create-personal"
    );
    incremental(before, work(&fixture.world.runtime));
    assert_eq!(managed(&fixture, &stranger), ordinary(&fixture, &stranger));
    assert_eq!(managed(&fixture, &stranger).len(), 1);
    let historical = request
        .at(&old)
        .query(queries::accounts())
        .execute()
        .unwrap();
    assert!(historical.rows().is_empty());
    let current = request.retain_current_read().unwrap();
    let denied = request
        .at(&current)
        .query(queries::current_accounts())
        .execute()
        .err()
        .expect("retained equal head cannot mint current selection");
    assert!(
        matches!(denied, WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported)
    );
}

#[test]
fn business_creation_updates_its_warm_owner_source_through_real_business_path() {
    let fixture = business_creation_world();
    let owner = fixture.owner();
    let teller = fixture.teller();
    let initial = fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(&owner)
        .controls(controls())
        .execute()
        .unwrap();
    assert!(initial.rows().is_empty());
    let before = work(&fixture.world.runtime);
    mutate!(
        fixture,
        &teller,
        mutations::create_business_account(CreateBusinessAccount {
            institution: fixture.institution,
            business: fixture.business,
            display_name: AccountName::new("Managed new business").unwrap(),
        }),
        "managed-create-business"
    );
    incremental(before, work(&fixture.world.runtime));
    let ordinary = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&owner)
        .controls(controls())
        .execute()
        .unwrap();
    let actual = fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(&owner)
        .controls(controls())
        .execute()
        .unwrap();
    assert_eq!(actual.rows(), ordinary.rows());
    assert_eq!(actual.rows().len(), 1);
}
