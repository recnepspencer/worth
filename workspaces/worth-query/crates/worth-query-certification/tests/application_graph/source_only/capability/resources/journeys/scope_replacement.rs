//! Actual same-name subject replacement cannot refresh a retained selector.
use super::*;

pub(super) fn run() {
    let host = published();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let resources = request.resources();
    let mut original = start(&resources, 16);
    let receipt = original.receipt();
    let metadata = metadata::admit(&host, &actor, &scope).unwrap();
    let identity = || {
        host.application
            .runtime()
            .with_application_advancement(&scope, |phase| {
                let selected = host
                    .application
                    .runtime()
                    .on_branch(host.application.runtime().current_world())
                    .select(&phase)
                    .unwrap();
                let parts = selected
                    .resolve_entity(
                        &phase,
                        DocumentIdentityField::reference(),
                        DOCUMENT_IDENTITY.to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
                    .relational_record_identity_parts();
                (parts.partition_id(), parts.local_slot(), parts.generation())
            })
            .unwrap()
    };
    let before = identity();
    for (restore, key) in [(false, 0x9181_8012_u64), (true, 0x9181_8013_u64)] {
        let changed = request
            .advanced()
            .mutate(super::super::lifecycle::Change { restore })
            .without_source()
            .idempotency(&key)
            .execute_in_program(
                &host.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    let after = identity();
    assert_ne!(
        before, after,
        "full genuine scope identities: {before:?} → {after:?}"
    );
    let contacts = host.contacts.resource.counts();
    let clocks = host.contacts.resource_clock.observations();
    assert!(resources.advance(&mut original).is_err());
    assert!(resources
        .result_available_governed(&receipt, metadata)
        .is_err());
    assert_eq!(host.contacts.resource.counts(), contacts);
    assert_eq!(host.contacts.resource_clock.observations(), clocks);
    // No generation-only slot-reuse claim: the original business Product may
    // retain that old slot. The full original subject still must not change.
    // Missing original scope leaves exact custody pending until process expiry;
    // dropping this fixture cannot attest to Native/provider cleanup.
}
