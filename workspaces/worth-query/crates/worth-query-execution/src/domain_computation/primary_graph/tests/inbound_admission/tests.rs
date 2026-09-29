use sha2::{Digest, Sha256};

use super::fixture::{installed_world, InboundWorld};
use super::verifier::signed_envelope;
use crate::domain_computation::application_aftermath::WorthQueryInboundVerificationDenial;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryExternalDispatchAdmissionDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundReceiptPosture as Posture,
};

pub(super) fn owner_commits(world: &InboundWorld) -> usize {
    world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| runtime.history().immutable_commit_count())
}

pub(super) fn product_commit(
    world: &InboundWorld,
) -> worth_runtime_world::facade::CompositeCommitIdentity {
    world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap()
        .selected_commit()
        .clone()
}

pub(super) fn completion_records(world: &InboundWorld) -> usize {
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let kind = world
        .application
        .primary_provider
        .graph
        .layout
        .provider_inbound_completion()
        .kind;
    world
        .application
        .primary_provider
        .graph
        .with_runtime_mut(|runtime| {
            let snapshot = super::super::super::exact_basis_access::open_exact_basis_snapshot(
                runtime,
                product.relational_basis(),
            )
            .unwrap();
            let count = runtime
                .read_truth()
                .project_snapshot(&snapshot)
                .unwrap()
                .bounded_entities_of_kind(kind, 100)
                .unwrap()
                .records()
                .len();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            count
        })
}

#[test]
fn installed_verifier_completes_a_genuine_dispatch_once_and_replays_its_terminal_result() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(11, "notice-one");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0x91; 32], record.payload(), false);
    let before_relational = owner_commits(&world);
    let before_product = product_commit(&world);
    assert_eq!(completion_records(&world), 0);
    let request = super::super::fixture::live_scope();
    let first = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .unwrap();
    assert_eq!(first.posture(), Posture::Performed);
    assert_eq!(first.message_identity(), &[0x91; 32]);
    let envelope_digest: [u8; 32] = Sha256::digest(&envelope).into();
    assert_eq!(first.envelope_digest(), &envelope_digest);
    assert_eq!(owner_commits(&world), before_relational + 1);
    assert_ne!(product_commit(&world), before_product);
    assert_eq!(completion_records(&world), 1);
    let completed_product = product_commit(&world);

    let replay = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .unwrap();
    assert_eq!(replay.posture(), Posture::Performed);
    assert_eq!(replay.message_identity(), first.message_identity());
    assert_eq!(owner_commits(&world), before_relational + 1);
    assert_eq!(product_commit(&world), completed_product);
    assert_eq!(completion_records(&world), 1);
}

#[test]
fn performed_inbound_completion_closes_later_dispatch_admission() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(21, "notice-terminal");
    let committed = world
        .application
        .observe_committed_dispatch_outbox(&dispatch)
        .unwrap()
        .unwrap();
    let envelope = signed_envelope(
        dispatch.dispatch_outbox().unwrap(),
        [0x95; 32],
        dispatch.dispatch_outbox().unwrap().payload(),
        false,
    );
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    assert!(matches!(
        world.application.admit_external_dispatch_attempt(committed),
        Err(WorthQueryExternalDispatchAdmissionDenial::AlreadyCompleted)
    ));
}

#[test]
fn lost_terminal_index_denies_until_bounded_owner_reconstruction() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(31, "notice-index-rebuild");
    let record = dispatch.dispatch_outbox().unwrap();
    let correlation = *record.correlation();
    let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let provider = &world.application.primary_provider;
    let sealed = provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .expect("World Performed must seal canonical terminal history");
    assert_ne!(
        sealed.original_world_commit(),
        sealed.completion_world_commit()
    );
    provider.destroy_completed_inbound_index_for_test();
    assert!(matches!(
        provider.lookup_completed_inbound(&correlation),
        Err(super::super::super::provider::WorthQueryInboundTerminalIndexDenial::IndexUnavailable)
    ));
    let one = std::num::NonZeroUsize::new(1).unwrap();
    let records = std::num::NonZeroUsize::new(32).unwrap();
    let ancestry = std::num::NonZeroUsize::new(32).unwrap();
    assert!(matches!(
        world.application.rebuild_completed_inbound_index(
            std::num::NonZeroUsize::new(32).unwrap(), records, one,
        ),
        Err(super::super::super::provider::WorthQueryInboundTerminalIndexDenial::ReconstructionWorkExhausted)
    ));
    assert!(provider.lookup_completed_inbound(&correlation).is_err());
    let mut pages = 0;
    loop {
        pages += 1;
        let complete = world
            .application
            .rebuild_completed_inbound_index(one, records, ancestry)
            .expect("each explicit World page is bounded and authoritative");
        if complete {
            break;
        }
        assert!(provider.lookup_completed_inbound(&correlation).is_err());
        assert!(
            pages < 32,
            "small fixture must complete within finite page budget"
        );
    }
    assert!(
        pages > 1,
        "one slot cannot silently scan the whole World history"
    );
    assert_eq!(
        provider.lookup_completed_inbound(&correlation).unwrap(),
        Some(sealed)
    );
}

#[test]
fn indexed_completion_row_reconstructs_only_with_original_world_history() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(32, "notice-durable-rebuild");
    let record = dispatch.dispatch_outbox().unwrap();
    let correlation = *record.correlation();
    let envelope = signed_envelope(record, [0x97; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let row = world
        .application
        .primary_provider
        .lookup_inbound_completion_row(product.relational_basis(), &correlation)
        .unwrap()
        .expect("indexed Relational completion row exists");
    assert_eq!(row.correlation, correlation);
    assert_eq!(row.original_commit, *dispatch.commit_reference());
    assert_ne!(row.completion_commit, row.original_commit);
    let reconstructed = world
        .application
        .reconstruct_completed_inbound_on_original(
            &correlation,
            product.observation().lifecycle_incarnation(),
            std::num::NonZeroUsize::new(32).unwrap(),
        )
        .unwrap()
        .expect("World ancestry pairs the indexed row");
    let sealed = world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .unwrap();
    assert_eq!(
        reconstructed.original_world_commit(),
        sealed.original_world_commit()
    );
    assert_eq!(
        reconstructed.completion_world_commit(),
        sealed.completion_world_commit()
    );
    assert_eq!(
        reconstructed.completion_attempt(),
        sealed.completion_attempt()
    );
    world
        .application
        .verify_completed_inbound_for_cleanup(&[correlation], u64::MAX)
        .expect("World-paired durable row permits exact expiry turnover");
    assert_eq!(
        world
            .application
            .primary_provider
            .lookup_completed_inbound(&correlation)
            .unwrap(),
        Some(sealed.clone())
    );
    world
        .application
        .verify_completed_inbound_for_cleanup(&[correlation], u64::MAX)
        .expect("retry after custody turnover remains idempotent");
}

#[test]
fn authenticated_payload_and_domain_mismatch_are_denied_before_custody() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(12, "notice-two");
    let record = dispatch.dispatch_outbox().unwrap();
    let before_relational = owner_commits(&world);
    let before_product = product_commit(&world);
    let request = super::super::fixture::live_scope();

    let wrong_domain = signed_envelope(record, [0x92; 32], record.payload(), true);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &wrong_domain, &request),
        Err(Denial::IncompatibleMeaning)
    ));
    let wrong_payload = signed_envelope(record, [0x92; 32], b"other-notice", false);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &wrong_payload, &request),
        Err(Denial::UnsupportedOutbox)
    ));
    let mut tampered = signed_envelope(record, [0x92; 32], record.payload(), false);
    *tampered.last_mut().unwrap() ^= 1;
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &tampered, &request),
        Err(Denial::Verification(
            WorthQueryInboundVerificationDenial::AuthenticationFailed
        ))
    ));
    assert_eq!(owner_commits(&world), before_relational);
    assert_eq!(product_commit(&world), before_product);
    assert_eq!(completion_records(&world), 0);

    let valid = signed_envelope(record, [0x92; 32], record.payload(), false);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &valid, &request)
            .unwrap()
            .posture(),
        Posture::Performed,
        "rejected signed envelopes must not occupy the one accepted slot"
    );
}

#[test]
fn finite_accepted_custody_denies_a_second_lawful_completion_before_ack() {
    let world = installed_world();
    let first = world.commit_dispatch(13, "notice-three");
    let first_record = first.dispatch_outbox().unwrap();
    let first_envelope = signed_envelope(first_record, [0x93; 32], first_record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &first_envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let second = world.commit_dispatch(14, "notice-four");
    let second_record = second.dispatch_outbox().unwrap();
    let second_envelope =
        signed_envelope(second_record, [0x94; 32], second_record.payload(), false);
    let before_relational = owner_commits(&world);
    let before_product = product_commit(&world);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &second_envelope, &request),
        Err(Denial::CapacityExhausted)
    ));
    assert_eq!(owner_commits(&world), before_relational);
    assert_eq!(product_commit(&world), before_product);
    assert_eq!(completion_records(&world), 1);
}

#[test]
fn retired_and_revoked_sources_cannot_admit_new_completion() {
    for revoked in [false, true] {
        let world = installed_world();
        let dispatch = world.commit_dispatch(31, "notice-source-state");
        let record = dispatch.dispatch_outbox().unwrap();
        let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
        if revoked {
            world
                .application
                .revoke_inbound_occurrence_source(&world.verifier)
                .unwrap();
        } else {
            world
                .application
                .retire_inbound_occurrence_source(&world.verifier)
                .unwrap();
        }
        let request = super::super::fixture::live_scope();
        let expected = if revoked {
            Denial::SourceRevoked
        } else {
            Denial::SourceRetired
        };
        assert!(matches!(
            world
                .application
                .receive_inbound_occurrence(&world.verifier, &envelope, &request),
            Err(actual) if actual == expected
        ));
        assert_eq!(completion_records(&world), 0);
    }
}
