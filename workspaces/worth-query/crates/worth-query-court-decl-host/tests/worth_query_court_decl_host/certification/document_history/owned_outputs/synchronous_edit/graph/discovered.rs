//! Discovered roots and their children must exist in the sole source publication.
use super::*;
mod failure;
mod permission;
mod program;
mod read_only_delivery;
mod support;
mod typed_refusal;
fn exercise(worker: bool, cold: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let mut host = world::install_program_with_other_model::<program::Program>(limits, 2, Some(3));
    let scope = request_scope();
    let observation = source_projection::Observation::begin();
    let retention = SynchronousCaptureRetentionForTest::install_on_this_thread();
    let mut identities = None;
    for (ordinal, value, expected_contacts) in [(0, 7, 4), (1, 9, 2), (2, 9, 0)] {
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let discoveries = observation.model_reads();
        let before = courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let outcome = request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value,
            })
            .without_source()
            .idempotency(&format!("discovered-edit-{ordinal}"))
            .execute_performed_discovered::<program::Program, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let done = match outcome {
            WorthQueryApplicationDiscoveredMutationOutcome::Performed(done) => done,
            WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(cause) => {
                panic!("discovered edit {ordinal} refused: {cause:?}")
            }
            WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied(cause) => panic!(
                "discovered edit {ordinal} lost outputs: {:?}",
                cause.denial()
            ),
            _ => panic!("discovered edit {ordinal} did not perform"),
        };
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts,
            "every synchronous root and descendant prepares before the source becomes visible"
        );
        assert_eq!(
            observation.model_reads(),
            discoveries + 1,
            "root discovery runs even when every producer is healthy"
        );
        let first_a = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let second_a = request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let first_b = request
            .query(query::first_output::Read("first:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let second_b = request
            .query(query::second_output::Read("second:b".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(
            [first_a.value, second_a.value, first_b.value, second_b.value],
            [value * 10, value + 1000, 30, 1003]
        );
        let ids = [
            first_a.native_id,
            second_a.native_id,
            first_b.native_id,
            second_b.native_id,
        ];
        if let Some(original) = identities {
            assert_eq!(ids, original);
        } else {
            identities = Some(ids);
        }
        assert_eq!(courts::native_head(&host).1, [before]);
        let head = courts::native_head(&host);
        let mut outputs = done
            .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
        let mut settled = false;
        for _ in 0..128 {
            if let WorthQueryDiscoveredProgramOutputProgress::Settled(result) = outputs
                .required_output_mut()
                .advance(&host, &request)
                .unwrap()
            {
                assert_eq!(result.root_outputs().count(), 2);
                settled = true;
                break;
            }
        }
        assert!(settled);
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + expected_contacts
        );
        assert_eq!(courts::native_head(&host), head);
        let funding = retention.counts();
        assert_eq!(funding.assemblies, ordinal + 1);
        assert_eq!(funding.missing_tickets, 0);
        assert_eq!(funding.mismatched_tickets, 0);
        if worker {
            assert_eq!(funding.unaccounted, ordinal + 1);
        } else {
            assert_eq!(funding.accounted, ordinal + 1);
            assert!(funding.container_bytes > 0);
            assert_eq!(funding.retained_bytes, funding.container_bytes);
        }

        if cold && ordinal == 0 {
            drop(outputs);
            let checkpoint = super::super::failure::checkpoint(&host);
            drop(request);
            drop(principal);
            drop(host);
            host = world::restore_program_checkpoint::<program::Program>(checkpoint, worker, false);
        }
    }
}
#[test]
fn discovery_and_all_descendants_prepare_at_d_in_the_one_edit_publication() {
    exercise(false, false)
}
#[test]
fn worker_only_discovery_and_all_descendants_prepare_at_d_in_the_one_edit_publication() {
    exercise(true, false)
}

#[test]
fn cold_discovered_roots_and_descendants_keep_their_original_identities() {
    exercise(false, true)
}
#[test]
fn worker_only_cold_discovered_roots_and_descendants_keep_their_original_identities() {
    exercise(true, true)
}
