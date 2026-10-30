//! Fresh capability/READ proofs precede every root lookup or callback.
use super::super::super::authentication;
use super::super::{host, prefix};
use super::{
    access,
    coordinates::Coordinates,
    geometry,
    provider::{BoxIndex, Calls},
    query, status,
};
use std::sync::Arc;
use worth_query_host::facade::{
    application_entry::*, application_installation::*, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};
fn input(directory: &str) -> query::FindGovernedBoxes {
    query::FindGovernedBoxes {
        directory: directory.into(),
        probe: Coordinates::new(0, 0, 4, 4),
    }
}
fn access(directory: &str) -> access::ReadAccess {
    access::ReadAccess {
        directory: directory.into(),
    }
}
fn revoked(error: WorthQueryApplicationRequestQueryDenial) {
    let WorthQueryApplicationRequestQueryDenial::CapabilityAdmission(cause) = error else {
        panic!("actual current capability refusal: {error:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing
        )
    );
}
#[test]
fn governed_box_provider_rechecks_real_scope_and_revoked_grant_before_callbacks() {
    std::thread::Builder::new()
        .name("governed-box-consumer".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}
fn run() {
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(Arc::new(prefix::Calls::default()), Arc::clone(&calls));
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let before = calls.totals();
    let missing = request
        .query(input("boxes"))
        .open_index::<BoxIndex>()
        .err()
        .expect("no disclosure capability");
    assert!(
        matches!(missing,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired)
    );
    assert_eq!(calls.totals(), before);
    let wrong_scope = request
        .query(input("other-boxes"))
        .open_index_governed::<BoxIndex, _, _, _>(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .err()
        .expect("genuine capability is scoped elsewhere");
    assert!(
        matches!(wrong_scope,WorthQueryApplicationRequestQueryDenial::Admission(cause) if cause.kind()==WorthQueryApplicationQueryAdmissionDenialKind::DisclosureAuthorizationMismatch)
    );
    assert_eq!(calls.totals(), before);
    let index = request
        .query(input("boxes"))
        .open_index_governed::<BoxIndex, _, _, _>(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .unwrap();
    geometry::not_ready(
        request
            .query(input("boxes"))
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .err()
            .expect("Created refuses after real governance"),
    );
    assert_eq!(calls.totals(), before);
    request
        .query(input("boxes"))
        .rebuild_index_governed(
            &index,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .unwrap();
    let source = request
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    let current = request
        .query(input("boxes"))
        .execute_governed(
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .unwrap();
    assert_eq!(
        current.rows(),
        geometry::expected(source.rows(), Coordinates::new(0, 0, 4, 4))
    );
    // Capability remains active while the independently declared READ path closes.
    let read_closed = request
        .advanced()
        .mutate(status::SetStatus {
            identity: "box-access".into(),
            active: true,
            readable: false,
        })
        .without_source()
        .idempotency(&0x919b003u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        read_closed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let before = calls.totals();
    for error in [
        request
            .query(input("boxes"))
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .err()
            .expect("lawful capability, READ denied"),
        request
            .query(input("boxes"))
            .open_index_governed::<BoxIndex, _, _, _>(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .err()
            .expect("open still needs Parent READ"),
        request
            .query(input("boxes"))
            .rebuild_index_governed(
                &index,
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .expect_err("rebuild still needs Parent READ"),
    ] {
        let WorthQueryApplicationRequestQueryDenial::Admission(cause) = error else {
            panic!("actual Parent READ admission: {error:?}")
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
                )
            )
        );
    }
    assert_eq!(
        calls.totals(),
        before,
        "lawful capability cannot supply Parent READ"
    );
    let read_restored = request
        .advanced()
        .mutate(status::SetStatus {
            identity: "box-access".into(),
            active: true,
            readable: true,
        })
        .without_source()
        .idempotency(&0x919b004u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        read_restored,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    request
        .query(input("boxes"))
        .rebuild_index_governed(
            &index,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .unwrap();
    assert_eq!(
        request
            .query(input("boxes"))
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes")
            )
            .unwrap()
            .rows(),
        geometry::expected(source.rows(), Coordinates::new(0, 0, 4, 4))
    );
    let other_calls = Arc::new(Calls::default());
    let other = host::install_boxes(Arc::new(prefix::Calls::default()), Arc::clone(&other_calls));
    let other_adapter = authentication::admit(other.installed_schema());
    let other_principal = authentication::block_on(other_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let other_request = other
        .runtime()
        .request(&other_principal, &scope)
        .on_branch(other.current_world());
    let foreign = other_request
        .query(input("boxes"))
        .rebuild_index_governed(
            &index,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .expect_err("actual foreign installed/runtime owner");
    assert!(
        matches!(foreign,WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref cause) if matches!(cause.cause(),WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::ForeignAssociation)))
    );
    assert_eq!(other_calls.totals(), (0, 0, 0));
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let version_calls = Arc::new(Calls::default());
    let incompatible = host::try_install(
        Arc::new(prefix::Calls::default()),
        Arc::clone(&version_calls),
        true,
    )
    .err()
    .expect("genuinely incompatible executable version");
    assert!(
        matches!(incompatible,WorthQueryInMemoryApplicationDenial::Contributions(ref cause) if cause.kind()==WorthQueryPrimaryGraphInstallationDenialKind::ManagedIndexProviderMeaningMismatch)
    );
    assert_eq!(version_calls.totals(), (0, 0, 0));
    let before_head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let revoked_status = request
        .advanced()
        .mutate(status::SetStatus {
            identity: "box-access".into(),
            active: false,
            readable: true,
        })
        .without_source()
        .idempotency(&0x919b001u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        revoked_status,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let after_revoke = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert_ne!(before_head, after_revoke);
    let before = calls.totals();
    revoked(
        request
            .query(input("boxes"))
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .err()
            .expect("revoked warm query"),
    );
    revoked(
        request
            .query(input("boxes"))
            .open_index_governed::<BoxIndex, _, _, _>(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .err()
            .expect("revoked open"),
    );
    revoked(
        request
            .query(input("boxes"))
            .rebuild_index_governed(
                &index,
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .expect_err("revoked rebuild"),
    );
    assert_eq!(
        calls.totals(),
        before,
        "fresh capability refusal precedes callbacks/cache outcomes"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &after_revoke
    );
    let restored = request
        .advanced()
        .mutate(status::SetStatus {
            identity: "box-access".into(),
            active: true,
            readable: true,
        })
        .without_source()
        .idempotency(&0x919b002u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        restored,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    geometry::not_ready(
        request
            .query(input("boxes"))
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes"),
            )
            .err()
            .expect("publication made the old image cold"),
    );
    request
        .query(input("boxes"))
        .rebuild_index_governed(
            &index,
            access::ReadBoxes::reference(),
            access::InspectBoxes::reference(),
            access("boxes"),
        )
        .unwrap();
    assert_eq!(
        request
            .query(input("boxes"))
            .execute_governed(
                access::ReadBoxes::reference(),
                access::InspectBoxes::reference(),
                access("boxes")
            )
            .unwrap()
            .rows(),
        geometry::expected(source.rows(), Coordinates::new(0, 0, 4, 4))
    );
    index.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
}
