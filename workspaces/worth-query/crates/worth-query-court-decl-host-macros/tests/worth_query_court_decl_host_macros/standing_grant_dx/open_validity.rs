//! W10A open grant windows on a Directory Home: `until` (open below),
//! `from` (open above, here on local calendar days) and `unbounded()`. An
//! open side admits every instant on that side, a present side still fails
//! closed at the one trusted admission instant, and eligibility agrees with
//! admission throughout and after reopening.
use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;

use super::grants::{
    AuditCapability, AuditOp, ObserveCapability, ObserveOp, ReviewCapability, ReviewOp,
    WitnessCapability, WitnessOp,
};
use super::observers::{home, SiteRole};
use super::{at, micros, november, temp_home_path};

#[test]
fn open_windows_admit_past_their_missing_bound_and_fail_closed_at_a_present_one() {
    let review = SiteRole {
        capability: ReviewCapability::reference(),
        operation: ReviewOp::reference(),
    };
    let audit = SiteRole {
        capability: AuditCapability::reference(),
        operation: AuditOp::reference(),
    };
    let witness = SiteRole {
        capability: WitnessCapability::reference(),
        operation: WitnessOp::reference(),
    };
    let observe = SiteRole {
        capability: ObserveCapability::reference(),
        operation: ObserveOp::reference(),
    };
    let window = november();
    let path = temp_home_path("open-validity");

    // Before the window opens: only the closed review role is not yet valid.
    let clock = WorthQueryFixedRuntimeTimeSource::new(micros(window.valid_from, -1));
    let home = home(&path, &clock);
    assert!(review.agreed(&home, "alice", "north").is_some());
    assert_eq!(audit.agreed(&home, "alice", "north"), None, "open below");
    assert!(witness.agreed(&home, "alice", "north").is_some());
    assert_eq!(observe.agreed(&home, "alice", "north"), None);

    // Years after every bound: the roles open above still admit, and the
    // present upper bounds have expired.
    clock.set(at(2031, 6, 1, 12, 0));
    assert!(review.agreed(&home, "alice", "north").is_some());
    assert!(
        audit.agreed(&home, "alice", "north").is_some(),
        "past `until`"
    );
    assert_eq!(witness.agreed(&home, "alice", "north"), None, "open above");
    assert_eq!(observe.agreed(&home, "alice", "north"), None);
    drop(home);

    // The same contracts reinstall from durable state and decide alike.
    let home = super::observers::home(&path, &clock);
    assert_eq!(witness.agreed(&home, "alice", "south"), None);
    assert_eq!(observe.agreed(&home, "alice", "south"), None);
    assert!(audit.agreed(&home, "alice", "north").is_some());
    // An open window is still a grant: bob holds none of these roles.
    assert!(observe.agreed(&home, "bob", "north").is_some());
}
