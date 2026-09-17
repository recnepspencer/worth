//! R8.51 — ordinary commit pays nothing for aftermath when no external or
//! recovery work is required.
//!
//! Gate 8.2 discipline: prove the aftermath machinery is live and available,
//! then assert the ordinary path's external_dispatch / undo_admission /
//! redo_admission counters are exactly zero.

use std::sync::Arc;

use bank_domain::model::Money;
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::SendMoney;
use bank_server::{mutations, BankMutationControls};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

use super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
use crate::fixture::{ordinary_read_world, principal_id, OWNER, RECIPIENT};
use crate::support::request_scope;

#[test]
fn ordinary_commit_pays_zero_aftermath_slots_while_machinery_is_live() {
    let rail = spawn_rail();
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    let fixture = ordinary_read_world("ordinary-aftermath-cost", 0);
    fixture
        .world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("the bank installs its rail once per runtime");
    // Live-before-zero: aftermath external-effect transport is present.
    assert!(
        fixture.world.runtime.has_external_effect_transport(),
        "aftermath machinery must be installed before asserting zero cost"
    );

    let owner = fixture.authenticate(OWNER);
    let outcome = fixture
        .world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: fixture.personal_account,
            recipient: principal_id(RECIPIENT),
            amount: Money::from_minor(100).unwrap(),
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            request_scope(),
            BankIdempotencyKey::new("ordinary-aftermath-cost-send").unwrap(),
        ))
        .execute();
    let Ok(WorthQueryApplicationMutationOutcome::Committed { receipt, .. }) = &outcome else {
        panic!("the lawful transfer must commit: {outcome:?}");
    };
    assert!(
        receipt.changed_record_count() >= 4,
        "the zero-work claim must cover a genuinely broad money-movement commit"
    );

    assert!(
        receipt.dispatch_outbox().is_none(),
        "undeclared external effect writes no outbox"
    );
    assert!(receipt.external_dispatch().is_none());
    assert!(
        receipt.retained_preimage().is_none(),
        "an ordinary no-demand commit must build no footprint and scan no decision facts"
    );

    let phases = receipt.canonical_work();
    for (lane, work) in [
        ("external_dispatch", phases.external_dispatch()),
        ("undo_admission", phases.undo_admission()),
        ("redo_admission", phases.redo_admission()),
    ] {
        assert_eq!(
            work.basis_preparations(),
            0,
            "{lane} must pay zero basis preparations on ordinary commit"
        );
        assert_eq!(
            work.digest_derivations(),
            0,
            "{lane} must pay zero digest derivations on ordinary commit"
        );
        assert_eq!(
            work.digest_text_materializations(),
            0,
            "{lane} must pay zero text materializations on ordinary commit"
        );
    }
    assert!(
        transport.attempts().is_empty(),
        "installed rail must stay silent for undeclared effects"
    );
}
