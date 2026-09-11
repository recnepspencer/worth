use bank_server::{
    BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankAuthorizationDenialKind, BankReadControls,
};
use worth_query_host::facade::publication::domain_computation::{
    WorthQueryPublishedApplicationQueryReleasePosture,
    WorthQueryPublishedApplicationQueryResultBufferRelease,
};

use super::fixture::{ordinary_read_world, OWNER, STRANGER};
use crate::support::request_scope;

#[test]
fn installed_account_activity_is_a_real_ordered_bank_query() {
    let fixture = ordinary_read_world("installed-account-activity", 0);
    let owner = fixture.authenticate(OWNER);
    let result = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&owner)
        .execute(current_controls())
        .expect("account owner should execute the installed query");

    assert_eq!(result.rows().len(), 1);
    let activity = &result.rows()[0];
    assert_eq!(activity.account(), fixture.personal_account);
    assert_eq!(
        activity
            .entries()
            .iter()
            .map(|item| item.account_sequence().get())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        activity
            .entries()
            .iter()
            .map(|item| item.amount().minor_units())
            .collect::<Vec<_>>(),
        vec![10_000, -2_500]
    );

    let inspection = result.receipt().inspect();
    assert_eq!(inspection.result_count(), 1);
    assert!(inspection.ordinary_work_units() > 0);
    assert_eq!(inspection.publication_canonical_entries(), 0);
    assert_eq!(inspection.publication_sha256_compression_blocks(), 0);
    assert_eq!(inspection.publication_identity_text_materializations(), 0);
    assert!(inspection.terminal_resources_released());
    let release = inspection.terminal_release();
    assert_eq!(
        release.application_basis(),
        WorthQueryPublishedApplicationQueryReleasePosture::Released
    );
    assert_eq!(
        release.graph_read_basis(),
        WorthQueryPublishedApplicationQueryReleasePosture::Released
    );
    let WorthQueryPublishedApplicationQueryResultBufferRelease::Released {
        limit_bytes,
        peak_bytes,
    } = release.result_buffer()
    else {
        panic!("a published result must retain exact released-buffer evidence")
    };
    assert!(peak_bytes > 0);
    assert!(peak_bytes <= limit_bytes);
    assert_eq!(release.released_graph_capacity_reservation_count(), 1);
}

#[test]
fn installed_account_activity_denies_a_mapped_stranger_before_plan_authority() {
    let fixture = ordinary_read_world("installed-account-activity-stranger", 0);
    let stranger = fixture.authenticate(STRANGER);
    let outcome = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&stranger)
        .execute(current_controls());
    let denial = match outcome {
        Err(denial) => denial,
        Ok(_) => panic!("mapped stranger received account query authority"),
    };

    assert!(matches!(
        denial,
        BankApplicationQueryDenial::Admission(error)
            if error.kind()
                == BankApplicationQueryAdmissionDenialKind::Authorization(
                    BankAuthorizationDenialKind::PermissionDenied
                )
    ));
}

fn current_controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 64, 100_000).unwrap()
}
