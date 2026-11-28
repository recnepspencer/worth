//! Standing role admission on a Directory Home: rooted site revocation, UTC
//! and local-date validity at the admission instant, the calendar the Home
//! must carry, and the self-grant refusal at the link.
use crate::time::{controlled_at, unix_nanos};
use worth_query_host::facade::runtime::time::ControlledTime;
use worth_query_host::facade::{
    local_calendar::WorthQueryTimeZone,
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Denial, WorthQueryPrincipalResolutionMode,
    },
};

use super::grants::{
    ApproveCapability, ApproveOp, ReviewCapability, ReviewOp, SiteRequest, ACTIVE,
};
use super::program::{caller, open, QmsHome};
use super::schema::{AssignRole, MemberBinding, SetSiteStatus};
use super::{at, micros, november, temp_home_path};

/// Mountain time with its daylight rule; November is standard time (UTC-7).
fn denver() -> WorthQueryTimeZone {
    WorthQueryTimeZone::from_tz_variable("MST7MDT,M3.2.0,M11.1.0").expect("POSIX rule parses")
}

fn home(path: &std::path::Path, clock: &std::sync::Arc<ControlledTime>) -> QmsHome {
    open(path, clock.clone(), Some(denver()), november())
        .unwrap_or_else(|denial| panic!("QMS Home opens: {denial}"))
}

enum Role {
    Review,
    Approve,
}

/// `name`'s admission of `role` over `site`: `None` admits, else the denial.
fn admit(home: &QmsHome, name: &str, role: Role, site: &str) -> Option<Denial> {
    let app = home.runtime().runtime();
    let who = caller(home, name);
    let request = SiteRequest {
        site: site.to_owned(),
    };
    app.with_application_advancement(&who.scope, |phase| {
        let selected = app.on_branch(app.current_world()).select(&phase).unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &phase,
                &app.installed_schema()
                    .principal_binding(MemberBinding::reference())
                    .unwrap(),
                &who.principal,
                &who.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let schema = app.installed_schema();
        match role {
            Role::Review => selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &schema
                        .capability(ReviewCapability::reference(), ReviewOp::reference())
                        .unwrap(),
                    request,
                    &who.scope,
                )
                .err(),
            Role::Approve => selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &schema
                        .capability(ApproveCapability::reference(), ApproveOp::reference())
                        .unwrap(),
                    request,
                    &who.scope,
                )
                .err(),
        }
        .map(|denial| denial.kind())
    })
    .unwrap()
}

fn set_site(home: &QmsHome, site: &str, status: &str) {
    home.session(caller(home, "alice").principal)
        .run(SetSiteStatus {
            site: site.to_owned(),
            status: status.to_owned(),
        })
        .unwrap_or_else(|denial| panic!("site status commits: {denial:?}"));
}

#[test]
fn deactivating_a_site_revokes_its_standing_roles_without_touching_a_grant() {
    let path = temp_home_path("rooted");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    assert_eq!(admit(&home, "alice", Role::Review, "north"), None);
    // bob holds no grant: the standing path admits only an actual holder.
    assert!(admit(&home, "bob", Role::Review, "north").is_some());

    set_site(&home, "north", "inactive");
    let revoked = admit(&home, "alice", Role::Review, "north");
    assert!(revoked.is_some(), "an inactive site revokes its grants");
    assert!(admit(&home, "alice", Role::Approve, "north").is_some());
    assert_eq!(admit(&home, "alice", Role::Review, "south"), None);
    drop(home);

    // The revocation is durable state, read again after reopening.
    let home = self::home(&path, &clock);
    assert_eq!(admit(&home, "alice", Role::Review, "north"), revoked);
    set_site(&home, "north", ACTIVE);
    assert_eq!(admit(&home, "alice", Role::Review, "north"), None);
}

#[test]
fn utc_validity_is_inclusive_at_the_admission_instant_across_reopen() {
    let path = temp_home_path("utc");
    let window = november();
    let clock = controlled_at(window.valid_until);
    let home = home(&path, &clock);
    assert_eq!(admit(&home, "alice", Role::Review, "north"), None);
    clock.set_wall_unix_nanos(unix_nanos(micros(window.valid_until, 1)));
    let expired = admit(&home, "alice", Role::Review, "north");
    assert!(expired.is_some(), "one microsecond past the bound expires");
    clock.set_wall_unix_nanos(unix_nanos(micros(window.valid_from, -1)));
    assert!(admit(&home, "alice", Role::Review, "north").is_some());
    drop(home);

    clock.set_wall_unix_nanos(unix_nanos(micros(window.valid_until, 1)));
    let home = self::home(&path, &clock);
    assert_eq!(admit(&home, "alice", Role::Review, "north"), expired);
    clock.set_wall_unix_nanos(unix_nanos(window.valid_from));
    assert_eq!(admit(&home, "alice", Role::Review, "north"), None);
}

#[test]
fn local_date_validity_turns_at_the_calendar_midnight_not_utc_midnight() {
    let path = temp_home_path("local-date");
    // 2026-11-27T06:59:59.999999Z is still November 26 in Denver (UTC-7).
    let local_midnight = at(2026, 11, 27, 7, 0);
    let clock = controlled_at(micros(local_midnight, -1));
    let home = home(&path, &clock);
    assert_eq!(admit(&home, "alice", Role::Approve, "north"), None);
    clock.set_wall_unix_nanos(unix_nanos(local_midnight));
    assert!(admit(&home, "alice", Role::Approve, "north").is_some());
    // The UTC review role is already past its bound at the same instant.
    assert!(admit(&home, "alice", Role::Review, "north").is_some());
    drop(home);

    clock.set_wall_unix_nanos(unix_nanos(micros(local_midnight, -1)));
    let home = self::home(&path, &clock);
    assert_eq!(admit(&home, "alice", Role::Approve, "south"), None);
}

#[test]
fn a_home_installing_local_date_roles_refuses_to_open_without_a_calendar() {
    let path = temp_home_path("no-calendar");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let Err(refused) = open(&path, clock, None, november()) else {
        panic!("a local-date role needs the Home's calendar");
    };
    assert!(
        format!("{refused:?}").contains("CapabilityCalendarMissing"),
        "{refused:?}"
    );
}

#[test]
fn an_operation_cannot_link_its_own_principal_as_a_role_holder() {
    let path = temp_home_path("self-grant");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    let assign = |member: u64, grant: &str| {
        home.session(caller(&home, "alice").principal)
            .run(AssignRole {
                site: "north".to_owned(),
                member,
                grant: grant.to_owned(),
            })
    };
    let refused = assign(1, "alice-self").expect_err("alice cannot grant herself");
    assert!(
        format!("{refused:?}").contains("CurrentAuthorityDenied")
            && format!("{refused:?}").contains("self-grant:RoleHolder"),
        "{refused:?}"
    );
    assign(2, "bob-review").unwrap_or_else(|denial| panic!("alice grants bob: {denial:?}"));
    assert_eq!(admit(&home, "bob", Role::Review, "north"), None);
    drop(home);

    let home = self::home(&path, &clock);
    assert_eq!(admit(&home, "bob", Role::Review, "north"), None);
    assert!(admit(&home, "bob", Role::Review, "south").is_some());
}
