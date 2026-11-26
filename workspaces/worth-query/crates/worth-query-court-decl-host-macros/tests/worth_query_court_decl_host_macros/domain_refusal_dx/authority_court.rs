//! Authority and session refusals fire before any handler or invariant runs,
//! so a caller who may not act learns nothing about the business rule.
use std::time::Duration;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionTimeController,
    WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal,
};

use super::program::{open, principal};
use super::schema::{ReceiveLot, ShipLot};
use super::{lot, temp_home_path, RunDenial};

/// Both requests would be refused by a rule (an expired lot, an overfull bin).
fn rule_breaking_runs(session_run: impl Fn(bool) -> RunDenial) -> [RunDenial; 2] {
    [session_run(true), session_run(false)]
}

fn assert_no_rule_disclosed(denial: &RunDenial, category: Category) {
    assert_eq!(denial.refusal_category(), category, "{denial:?}");
    assert!(denial.domain_refusal().is_none(), "{denial:?}");
    let summary = denial.user_summary();
    assert!(summary.domain_refusal().is_none());
    assert_eq!(summary.user_message(), category.default_message());
}

#[test]
fn a_revoked_principal_is_refused_permission_without_learning_the_rule() {
    let time = WorthQueryAdmissionTimeController::new().expect("controlled clock");
    let _held = time.hold();
    let clock = time.clock();
    let home = open(&temp_home_path("authority"), &clock);
    let before = home.committed_basis_version();
    let revoked = home.session(principal(&home, &clock, "revoked"));
    let denials = rule_breaking_runs(|ship| {
        if ship {
            revoked
                .run(ShipLot {
                    lot: 2,
                    quantity: 1,
                })
                .expect_err("revoked")
        } else {
            revoked
                .run(ReceiveLot {
                    lot: 1,
                    quantity: 500,
                })
                .expect_err("revoked")
        }
    });
    for denial in &denials {
        assert_no_rule_disclosed(denial, Category::PermissionDenied);
    }
    // The owner, by contrast, is told which rule refused.
    let owner = home.session(principal(&home, &clock, "owner"));
    let ship = owner
        .run(ShipLot {
            lot: 2,
            quantity: 1,
        })
        .expect_err("expired");
    assert!(ship.domain_refusal().is_some());

    // An expired proof refuses before the rule as well, even for the owner.
    time.advance(Duration::from_secs(601))
        .expect("clock advances");
    let denials = rule_breaking_runs(|ship| {
        if ship {
            owner
                .run(ShipLot {
                    lot: 2,
                    quantity: 1,
                })
                .expect_err("expired proof")
        } else {
            owner
                .run(ReceiveLot {
                    lot: 1,
                    quantity: 500,
                })
                .expect_err("expired proof")
        }
    });
    for denial in &denials {
        assert_no_rule_disclosed(denial, Category::SessionExpired);
    }
    drop((revoked, owner));
    // None of the refused runs committed; a fresh proof reads the lots back.
    assert!(!home.committed_basis_version().changed_since(before));
    assert_eq!(
        (
            lot(&home, &clock, 1).quantity,
            lot(&home, &clock, 2).quantity
        ),
        (40, 12)
    );
    home.close_clean().expect("Home closes");
}
