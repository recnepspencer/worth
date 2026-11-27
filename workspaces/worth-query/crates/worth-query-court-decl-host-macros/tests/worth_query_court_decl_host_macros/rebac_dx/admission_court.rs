//! Admission decisions of the role-matrix rules, before and after reopen.
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryAdmissionTimeController,
    },
    WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal,
};

use super::program::{open, principal, RebacHome};
use super::schema::{AnnotateCase, CloseCase, ReadCase};
use super::{case, temp_home_path, RunDenial};

fn close(
    home: &RebacHome,
    clock: &WorthQueryAdmissionClock,
    user: &str,
    case: u64,
) -> Result<(), RunDenial> {
    home.session(principal(home, clock, user))
        .run(CloseCase {
            case,
            note: 900 + case,
        })
        .map(|_| ())
}

fn annotate(
    home: &RebacHome,
    clock: &WorthQueryAdmissionClock,
    user: &str,
    case: u64,
    note: u64,
) -> Result<(), RunDenial> {
    home.session(principal(home, clock, user))
        .run(AnnotateCase { case, note })
        .map(|_| ())
}

fn assert_permission_denied(result: Result<(), RunDenial>, why: &str) {
    match result {
        Err(denial) => assert_eq!(
            denial.refusal_category(),
            Category::PermissionDenied,
            "{why}: {denial:?}"
        ),
        Ok(()) => panic!("{why}: the run was admitted"),
    }
}

/// Close needs the Manager role at the case's site, level >= 2, and not authorship.
#[test]
fn role_site_level_and_author_edge_decide_who_closes() {
    let time = WorthQueryAdmissionTimeController::new().expect("controlled clock");
    let _held = time.hold();
    let clock = time.clock();
    let home = open(&temp_home_path("close"), &clock);
    let before = home.committed_basis_version();
    assert_permission_denied(close(&home, &clock, "reviewer", 10), "Reviewer lacks Close");
    assert_permission_denied(close(&home, &clock, "junior", 10), "level 1 is below 2");
    assert_permission_denied(
        close(&home, &clock, "elsewhere", 10),
        "assignment is at site 2",
    );
    assert_permission_denied(
        close(&home, &clock, "author", 11),
        "the author may not close",
    );
    assert!(
        !home.committed_basis_version().changed_since(before),
        "refusals commit nothing"
    );
    assert_eq!(case(&home, &clock, 11).note, 0);

    close(&home, &clock, "manager", 10).expect("a level-2 manager at site 1 closes");
    close(&home, &clock, "author", 10).expect("the author of case 11 closes case 10");
    assert_eq!(case(&home, &clock, 10).note, 910);
    home.close_clean().expect("Home closes");
}

/// Annotate compares the case's recorded author field with the actor's user id.
#[test]
fn the_recorded_author_may_not_annotate_but_others_holding_the_role_may() {
    let time = WorthQueryAdmissionTimeController::new().expect("controlled clock");
    let _held = time.hold();
    let clock = time.clock();
    let home = open(&temp_home_path("annotate"), &clock);
    assert_permission_denied(
        annotate(&home, &clock, "reviewer", 10, 1),
        "reviewer authored 10",
    );
    assert_permission_denied(
        annotate(&home, &clock, "auditor", 10, 1),
        "Auditor lacks Annotate",
    );
    annotate(&home, &clock, "reviewer", 11, 21).expect("the reviewer annotates another's case");
    annotate(&home, &clock, "manager", 10, 22).expect("the manager annotates case 10");
    assert_eq!(
        (case(&home, &clock, 10).note, case(&home, &clock, 11).note),
        (22, 21)
    );
    home.close_clean().expect("Home closes");
}

/// The gated read and every rule decide identically after a Directory reopen,
/// which re-admits the extended policy paths from the persisted archive.
#[test]
fn reads_require_the_ability_and_every_decision_survives_reopen() {
    let time = WorthQueryAdmissionTimeController::new().expect("controlled clock");
    let _held = time.hold();
    let clock = time.clock();
    let path = temp_home_path("reopen");
    let home = open(&path, &clock);
    let read = |home: &RebacHome, user: &str| {
        home.session(principal(home, &clock, user))
            .query(ReadCase { case: 10 })
            .map(|rows| rows.len())
    };
    assert_eq!(
        read(&home, "auditor").ok(),
        Some(1),
        "Auditor reads its site's case"
    );
    let refused = read(&home, "elsewhere").expect_err("site 2 may not read a site-1 case");
    assert_eq!(
        refused.refusal_category(),
        Category::PermissionDenied,
        "{refused:?}"
    );
    home.close_clean().expect("Home closes");

    let home = open(&path, &clock);
    assert_eq!(read(&home, "auditor").ok(), Some(1));
    assert!(read(&home, "elsewhere").is_err());
    assert_permission_denied(
        close(&home, &clock, "junior", 10),
        "ordered predicate after reopen",
    );
    assert_permission_denied(
        close(&home, &clock, "author", 11),
        "author edge after reopen",
    );
    assert_permission_denied(
        annotate(&home, &clock, "reviewer", 10, 3),
        "actor field after reopen",
    );
    close(&home, &clock, "manager", 10).expect("the manager still closes after reopen");
    home.close_clean().expect("Home closes");
}
