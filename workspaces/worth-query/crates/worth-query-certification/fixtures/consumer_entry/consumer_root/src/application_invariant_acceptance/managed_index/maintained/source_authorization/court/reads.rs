//! Both real Source completion modes refuse READ independently of Public Parent.
use super::*;
#[test]
fn native_source_read_policy_revoke_keeps_structural_root_and_denies_before_callbacks() {
    lifecycle::run(|| {
        let setup = Setup::new();
        let request = setup
            .world
            .runtime()
            .request(&setup.principal, &setup.scope)
            .on_branch(setup.world.current_world());
        let original = request.query(native()).execute().unwrap();
        assert_eq!(
            original
                .rows()
                .iter()
                .map(|row| row.identity.as_str())
                .collect::<Vec<_>>(),
            ["authorized-item"]
        );
        let basis = request.retain_current_read().unwrap();
        let before = callbacks(&setup.calls.native);
        let denial = request
            .at(&basis)
            .query(native())
            .execute()
            .err()
            .expect("same-head pinned is unsupported");
        assert!(
            matches!(denial, WorthQueryApplicationRequestQueryDenial::Admission(cause)
            if cause.kind() == WorthQueryApplicationQueryAdmissionDenialKind::BasisUnsupported)
        );
        assert_eq!(callbacks(&setup.calls.native), before);
        setup.set_readable(false, 0x919c_0010);
        let denial = request
            .query(native())
            .execute()
            .err()
            .expect("Source READ was revoked");
        let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = denial else {
            panic!("{denial:?}")
        };
        assert!(
            matches!(cause.cause(), WorthQueryManagedIndexDenialCause::QueryAdmission(denial)
            if denial.kind() == WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::Refused(WorthQueryOperationAuthorizationRefusalKind::PermissionDenied))),
            "{cause:?}"
        );
        assert_eq!(callbacks(&setup.calls.native), before);
        assert_eq!(original.rows()[0].name, "shared");
        setup.set_readable(true, 0x919c_0011);
        assert_eq!(
            request.query(native()).execute().unwrap().rows(),
            original.rows()
        );
    });
}
#[test]
fn completed_source_ready_read_rechecks_original_policy_without_provider_callbacks() {
    lifecycle::run(|| {
        let setup = Setup::new();
        let request = setup
            .world
            .runtime()
            .request(&setup.principal, &setup.scope)
            .on_branch(setup.world.current_world());
        let index = request
            .query(completed())
            .open_index::<CompletedIndex>()
            .unwrap();
        request.query(completed()).rebuild_index(&index).unwrap();
        let original = request.query(completed()).execute().unwrap();
        assert_eq!(original.rows().len(), 1);
        let before = callbacks(&setup.calls.completed);
        setup.set_readable(false, 0x919c_0020);
        let denial = request
            .query(completed())
            .execute()
            .err()
            .expect("cached rows cannot replace Source READ permission");
        let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = denial else {
            panic!("{denial:?}")
        };
        assert!(
            matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::SourceVerification {
                    authorization: Some(WorthQueryOperationAuthorizationDenialKind::Refused(
                        WorthQueryOperationAuthorizationRefusalKind::StaleAuthorization
                    )),
                    ..
                }
            ),
            "{cause:?}"
        );
        assert_eq!(callbacks(&setup.calls.completed), before);
        setup.set_readable(true, 0x919c_0021);
        request.query(completed()).rebuild_index(&index).unwrap();
        assert_eq!(
            request.query(completed()).execute().unwrap().rows(),
            original.rows()
        );
        index.close();
    });
}
