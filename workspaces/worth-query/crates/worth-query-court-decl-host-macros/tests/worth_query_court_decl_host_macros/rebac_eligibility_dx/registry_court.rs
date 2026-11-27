//! A registry onboards a brand-new user on an open Home: authority is granted
//! through ordinary relationships, eligibility and execution follow, a
//! duplicate identity can never map twice, and all of it survives reopen.
use worth_query_decl::facade::application_schema::{ApplicationSchema, ApplicationSchemaMember};
use worth_query_host::facade::eligibility::WorthQueryAbilityEligibility as Verdict;
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;
use worth_query_host::facade::principal_registry::WorthQueryPrincipalRegistryRefusal as Refusal;
use worth_query_host::facade::{
    WorthQueryRefusalCategory, WorthQuerySessionRunDenial, WorthQueryUserFacingRefusal,
};

use super::program::{identity, open};
use super::*;

type RunDenial = WorthQuerySessionRunDenial<WorthQueryOperationBodyDenial>;

/// Frank is no seeded operator; he joins at runtime as operator 10.
const FRANK: (&str, u64) = ("frank", 10);

fn onboard(admin: &Session<'_>, subject: &str, operator: u64) -> Result<(), RunDenial> {
    admin
        .run(accounts::OnboardPrincipal {
            directory: SITE.to_owned(),
            identity: identity(subject),
            principal: operator,
        })
        .map(drop)
}

fn registry_refusal(denial: RunDenial) -> Option<Refusal> {
    Refusal::of(&denial.domain_refusal()?)
}

#[test]
fn a_user_onboarded_at_runtime_is_granted_eligible_acts_and_survives_reopen() {
    let path = temp_home_path("onboarding");
    let clock = WorthQueryAdmissionClock::system();
    {
        let home = open(&path, &clock);
        let admin = session(&home, &clock, "admin");
        let frank = session(&home, &clock, FRANK.0);
        assert_eq!(
            verdict(&frank, "D3"),
            Verdict::Unmapped,
            "not yet onboarded"
        );
        onboard(&admin, FRANK.0, FRANK.1).expect("the admin onboards frank");
        assert_eq!(
            verdict(&frank, "D3"),
            Verdict::Denied,
            "mapped, no authority yet"
        );
        assign(&admin, "frank-approves-d3", FRANK.1, "approver", "D3");
        assert_eq!(verdict(&frank, "D3"), Verdict::Eligible);
        let roster = session(&home, &clock, "alice")
            .eligible_among(
                &approve("D1"),
                ViewApprovers::reference(),
                &[identity(FRANK.0)],
            )
            .expect("alice sees D1's approvers");
        assert_eq!(roster.verdict(&identity(FRANK.0)), Some(Verdict::Denied));
        assert!(approves(&frank, "D3"), "frank's own session acts");
        home.close_clean().expect("the Home closes cleanly");
    }
    let home = open(&path, &clock);
    let frank = session(&home, &clock, FRANK.0);
    assert_eq!(
        verdict(&frank, "D3"),
        Verdict::Eligible,
        "authority survives reopen"
    );
    assert!(approves(&frank, "D3"));
}

#[test]
fn a_duplicate_identity_or_principal_is_refused_before_any_effect() {
    let path = temp_home_path("duplicate");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let admin = session(&home, &clock, "admin");
    onboard(&admin, FRANK.0, FRANK.1).expect("first onboarding commits");
    let (counter, _guard) = count_commits(&home);
    let again = onboard(&admin, FRANK.0, 11).expect_err("one identity maps once");
    assert_eq!(
        registry_refusal(again),
        Some(Refusal::IdentityAlreadyOnboarded)
    );
    // A seeded identity counts too: alice already maps.
    let seeded = onboard(&admin, "alice", 12).expect_err("alice is already mapped");
    assert_eq!(
        registry_refusal(seeded),
        Some(Refusal::IdentityAlreadyOnboarded)
    );
    let taken = onboard(&admin, "grace", FRANK.1).expect_err("operator 10 is frank");
    assert_eq!(
        registry_refusal(taken),
        Some(Refusal::PrincipalIdentityTaken)
    );
    assert_eq!(commits(&counter), 0, "refused onboardings leave no effect");
    // The registry declares the mapping identity unique: a duplicate cannot
    // be written by any path, not only refused by this handler.
    let declaration = <QmsSchema as ApplicationSchema>::declaration().expect("the schema declares");
    let identity = accounts::PrincipalAccountFacts::external_identity();
    assert!(declaration.erased().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::Field { entity, field, unique: true, .. }
            if entity == identity.entity() && field == identity.field()
    )));
}

#[test]
fn only_the_onboarding_ability_onboards_and_disabling_stops_acting() {
    let path = temp_home_path("onboarding-authority");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let alice = session(&home, &clock, "alice");
    let refused = onboard(&alice, FRANK.0, FRANK.1).expect_err("alice may not onboard");
    assert_eq!(
        refused.refusal_category(),
        WorthQueryRefusalCategory::PermissionDenied
    );
    assert_eq!(
        verdict(&session(&home, &clock, FRANK.0), "D1"),
        Verdict::Unmapped
    );
    let admin = session(&home, &clock, "admin");
    let status = |enabled: bool| {
        admin
            .run(accounts::SetPrincipalStatus {
                directory: SITE.to_owned(),
                identity: identity("alice"),
                enabled,
            })
            .expect("the admin sets alice's status")
    };
    status(false);
    assert_eq!(verdict(&alice, "D1"), Verdict::Disabled);
    assert!(!approves(&alice, "D1"), "a disabled account no longer acts");
    status(true);
    assert_eq!(verdict(&alice, "D1"), Verdict::Eligible);
    assert!(approves(&alice, "D1"));
}
