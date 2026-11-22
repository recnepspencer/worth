//! A residual where filter narrows account discovery by display name.
use super::*;

/// The fixture names the owner's personal account "Daily", the owner's
/// business account "Operations" and the recipient's account "Recipient".
#[test]
fn account_search_matches_display_name_ignoring_ascii_case_within_visible_accounts() {
    let fixture = ordinary_read_world("account-search", 2);
    let owner = fixture.authenticate(OWNER);
    let viewer = fixture.authenticate(VIEWER);
    let recipient = fixture.authenticate(RECIPIENT);

    assert_eq!(
        search(&fixture, &owner, "dAiLy"),
        [fixture.personal_account]
    );
    assert_eq!(search(&fixture, &owner, "OPER"), [fixture.business_account]);
    let mut both = vec![fixture.personal_account, fixture.business_account];
    both.sort();
    assert_eq!(search(&fixture, &owner, "a"), both);
    assert_eq!(
        search(&fixture, &recipient, "recip"),
        [fixture.recipient_account]
    );
    // A matching account outside the principal's discovery paths never leaks.
    assert!(search(&fixture, &owner, "recipient").is_empty());
    // An empty needle filters nothing: the search equals plain discovery.
    for actor in [&owner, &viewer, &recipient] {
        assert_eq!(
            search(&fixture, actor, ""),
            ordinary(&fixture, actor)
                .iter()
                .map(|row| row.id())
                .collect::<Vec<_>>()
        );
    }

    assert!(search(&fixture, &viewer, "operations").is_empty());
    mutate!(
        fixture,
        &owner,
        mutations::grant_account_access(GrantAccountAuthorization {
            account: fixture.business_account,
            principal: principal_id(VIEWER),
            role: CustomerRole::Viewer,
        }),
        "account-search-grant-business"
    );
    assert_eq!(
        search(&fixture, &viewer, "operations"),
        [fixture.business_account]
    );
}
