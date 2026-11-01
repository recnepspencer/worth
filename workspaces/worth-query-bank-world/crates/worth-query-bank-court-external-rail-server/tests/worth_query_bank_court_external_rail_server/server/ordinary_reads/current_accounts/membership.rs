//! Live source membership changes reach the actual incremental provider.
use super::*;
#[test]
fn warm_grant_revoke_and_remaining_owner_path_preserve_real_source_membership() {
    let fixture = ordinary_read_world("managed-account-membership", 0);
    let owner = fixture.authenticate(OWNER);
    let viewer = fixture.authenticate(VIEWER);
    assert_eq!(managed(&fixture, &viewer), ordinary(&fixture, &viewer));
    assert_eq!(managed(&fixture, &owner), ordinary(&fixture, &owner));
    let initial = work(&fixture.world.runtime);
    mutate!(
        fixture,
        &owner,
        mutations::grant_account_access(GrantAccountAuthorization {
            account: fixture.business_account,
            principal: principal_id(VIEWER),
            role: CustomerRole::Viewer,
        }),
        "managed-grant-business"
    );
    let granted = work(&fixture.world.runtime);
    incremental(initial, granted);
    assert_eq!(managed(&fixture, &viewer), ordinary(&fixture, &viewer));
    assert_eq!(managed(&fixture, &viewer).len(), 2);
    let id = authorization(&fixture, &owner, fixture.business_account, VIEWER);
    mutate!(
        fixture,
        &owner,
        mutations::revoke_account_access(RevokeAccountAuthorization {
            account: fixture.business_account,
            authorization: id,
        }),
        "managed-revoke-business"
    );
    incremental(granted, work(&fixture.world.runtime));
    assert_eq!(managed(&fixture, &viewer), ordinary(&fixture, &viewer));
    assert_eq!(managed(&fixture, &viewer).len(), 1);
    // Owner still reaches BusinessAccount through reverse BusinessOwner after
    // its independent authorization edge disappears. The union must not retire it.
    let id = authorization(&fixture, &owner, fixture.business_account, OWNER);
    let before = work(&fixture.world.runtime);
    mutate!(
        fixture,
        &owner,
        mutations::revoke_account_access(RevokeAccountAuthorization {
            account: fixture.business_account,
            authorization: id,
        }),
        "managed-revoke-owner-alternate-path"
    );
    assert_eq!(work(&fixture.world.runtime).0, before.0);
    assert_eq!(managed(&fixture, &owner), ordinary(&fixture, &owner));
    assert_eq!(managed(&fixture, &owner).len(), 2);
    let id = authorization(&fixture, &owner, fixture.personal_account, VIEWER);
    let before = work(&fixture.world.runtime);
    mutate!(
        fixture,
        &owner,
        mutations::revoke_account_access(RevokeAccountAuthorization {
            account: fixture.personal_account,
            authorization: id,
        }),
        "managed-revoke-final-viewer-path"
    );
    incremental(before, work(&fixture.world.runtime));
    assert!(managed(&fixture, &viewer).is_empty());
    assert!(ordinary(&fixture, &viewer).is_empty());
}
