//! W10A single source of truth: capability eligibility is read from the
//! installed capability contract, so it refuses exactly when capability
//! admission refuses. The audit role needs a current standing grant and an
//! enabled site assignment; each is withdrawn while the other stays intact,
//! and both observers must agree on the outcome and its policy cause.
use crate::time::{controlled_at, unix_nanos};

use worth_query_host::facade::{
    eligibility::{
        WorthQueryAbilityEligibilityDenial, WorthQueryCapabilityEligibility as Verdict,
        WorthQueryEligibilityDenial,
    },
    primary_graph::WorthQueryApplicationAuthorizationExplanationCause as Cause,
};

use worth_query_host::facade::{WorthQueryRefusalCategory, WorthQueryUserFacingRefusal};

use super::grants::{AuditCapability, AuditOp, ReviewCapability, ViewAuditors, ACTIVE, ENABLED};
use super::observers::{home, SiteRole};
use super::program::{caller, identity, QmsHome};
use super::schema::{SetAssignmentStatus, SetGrantStatus};
use super::{at, micros, november, temp_home_path};

fn audit() -> SiteRole<AuditCapability, AuditOp> {
    SiteRole {
        capability: AuditCapability::reference(),
        operation: AuditOp::reference(),
    }
}

fn agreed(home: &QmsHome, name: &str, site: &str) -> Option<Cause> {
    audit().agreed(home, name, site)
}

/// Enables or disables `name`'s north assignment; no grant is touched.
fn assignment(home: &QmsHome, name: &str, status: &str) {
    home.session(caller(home, "alice").principal)
        .run(SetAssignmentStatus {
            assignment: format!("{name}-north"),
            status: status.to_owned(),
        })
        .unwrap_or_else(|denial| panic!("assignment status commits: {denial:?}"));
}

/// Sets the north audit grant's status; no assignment is touched.
fn grant(home: &QmsHome, status: &str) {
    home.session(caller(home, "alice").principal)
        .run(SetGrantStatus {
            grant: "north-audit".to_owned(),
            status: status.to_owned(),
        })
        .unwrap_or_else(|denial| panic!("grant status commits: {denial:?}"));
}

#[test]
fn eligibility_and_admission_refuse_identically_when_either_half_is_withdrawn() {
    let path = temp_home_path("single-source");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    assert_eq!(agreed(&home, "alice", "north"), None);
    // bob is assigned but holds no grant; alice holds a south grant but no
    // south assignment. Neither half alone admits.
    assert!(agreed(&home, "bob", "north").is_some());
    assert!(agreed(&home, "alice", "south").is_some());

    // 1. Disable the assignment; the grant stays active and in validity.
    assignment(&home, "alice", "disabled");
    let disabled = agreed(&home, "alice", "north");
    assert!(disabled.is_some(), "a disabled assignment refuses both");
    assignment(&home, "alice", ENABLED);
    assert_eq!(agreed(&home, "alice", "north"), None);

    // 2. Revoke the grant; the assignment stays enabled.
    grant(&home, "revoked");
    assert_eq!(
        agreed(&home, "alice", "north"),
        Some(Cause::MissingCapability)
    );
    grant(&home, ACTIVE);
    assert_eq!(agreed(&home, "alice", "north"), None);

    // 3. Let the grant expire at the trusted instant; nothing is written.
    clock.set_wall_unix_nanos(unix_nanos(micros(november().valid_until, 1)));
    let expired = agreed(&home, "alice", "north");
    assert!(expired.is_some(), "an expired grant refuses both");
    drop(home);

    // The same verdicts hold on durable state after reopening.
    let home = self::home(&path, &clock);
    assert_eq!(agreed(&home, "alice", "north"), expired);
    clock.set_wall_unix_nanos(unix_nanos(november().valid_until));
    assert_eq!(agreed(&home, "alice", "north"), None);
}

#[test]
fn a_capability_roster_shares_the_ability_roster_disclosure_gate() {
    let path = temp_home_path("capability-roster");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    let staff = [identity("alice"), identity("bob"), identity("carol")];
    let roster = home
        .session(caller(&home, "alice").principal)
        .eligible_among_for_capability(
            &audit().question("north"),
            ViewAuditors::reference(),
            &staff,
        )
        .unwrap_or_else(|denial| panic!("alice holds a grant over north: {denial:?}"));
    let verdicts: Vec<_> = roster.iter().map(|(_, verdict)| verdict).collect();
    assert_eq!(
        verdicts,
        [
            Verdict::Eligible,
            Verdict::Denied(Cause::MissingCapability),
            Verdict::Unmapped,
        ]
    );
    assert_eq!(roster.eligible().collect::<Vec<_>>(), [&staff[0]]);

    // bob holds no grant over north, so he may not see who can audit it.
    let refused = home
        .session(caller(&home, "bob").principal)
        .eligible_among_for_capability(
            &audit().question("north"),
            ViewAuditors::reference(),
            &staff,
        )
        .expect_err("bob holds no role over north");
    assert!(
        matches!(
            refused,
            WorthQueryEligibilityDenial::Observation(
                WorthQueryAbilityEligibilityDenial::RosterNotDisclosed { .. }
            )
        ),
        "{refused:?}"
    );
}

#[test]
fn a_capability_paired_with_an_operation_it_is_not_installed_on_refuses_typed() {
    let path = temp_home_path("capability-not-installed");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    // Review is installed on ReviewOp only; AuditOp carries the same request.
    let mismatched = SiteRole {
        capability: ReviewCapability::reference(),
        operation: AuditOp::reference(),
    };
    let refused = home
        .session(caller(&home, "alice").principal)
        .eligible_for_capability(&mismatched.question("north"))
        .expect_err("no contract pairs Review with AuditOp");
    assert!(
        matches!(
            refused,
            WorthQueryEligibilityDenial::Observation(
                WorthQueryAbilityEligibilityDenial::CapabilityNotInstalled { .. }
            )
        ),
        "{refused:?}"
    );
    // A caller defect, not a missing grant: no permission prompt is shown.
    assert_eq!(
        refused.refusal_category(),
        WorthQueryRefusalCategory::Internal
    );
    // The installed pair is answered for the same principal and site.
    assert_eq!(
        SiteRole {
            capability: ReviewCapability::reference(),
            operation: super::grants::ReviewOp::reference(),
        }
        .eligibility(&home, "alice", "north"),
        Verdict::Eligible
    );
}
