//! Every publication retires the row of the generation its record displaces,
//! whether or not a demand manages it. The lineage never names a displaced
//! generation again, so the owner keeps a release it could not complete.

use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, idempotency,
    preimage_evidence::{retained_status_program, RetentionMutationBreadth},
    resolved_account,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

type Identity = Arc<RecordedSettlementIdentity>;

/// Runs `journey` over a world whose publications no demand owns: they carry
/// no prerequisites. `publish(status, replacement, key)` publishes one output
/// partition at the next product generation, and `live_rows` answers which
/// settlements the live mark root still holds.
fn with_unmanaged_publications(
    journey: impl FnOnce(
        &SourceInvalidationOwner,
        &dyn Fn(&str, &str, u8) -> Identity,
        &dyn Fn(&[&Identity]) -> Vec<bool>,
    ),
) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let partition = [7; 32];
    let publish = |status: &str, replacement: &str, key: u8| {
        let account = resolved_account(&world, status, &request);
        let program = retained_status_program(
            &world,
            &principal,
            &account,
            &request,
            replacement,
            RetentionMutationBreadth::Narrow,
        );
        let outcome = world.application.compare_and_commit_application(
            program,
            idempotency(key, key + 1).bind_source_partition(&partition),
        );
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            panic!("the output publication commits: {outcome:?}");
        };
        Arc::clone(
            receipt
                .exact_output_settlement()
                .expect("an output publication records its settlement"),
        )
    };
    let live_rows = |identities: &[&Identity]| {
        handle.with_runtime_mut(|runtime| {
            let (basis_handle, basis) = snapshot(runtime);
            let cell = owner
                .cell_for_read(&basis, &mut owner.edit_admission())
                .unwrap()
                .unwrap();
            let image = cell.read_image();
            let live = identities
                .iter()
                .map(|identity| image.payload().current.settlements.contains_key(*identity))
                .collect::<Vec<_>>();
            runtime.snapshots().release_snapshot(&basis_handle).unwrap();
            live
        })
    };
    journey(owner, &publish, &live_rows);
}

#[test]
fn a_publication_without_prerequisites_retires_the_row_its_generation_displaces() {
    with_unmanaged_publications(|_, publish, live_rows| {
        let first = publish("open", "frozen", 31);
        assert_eq!(live_rows(&[&first]), [true]);
        let second = publish("frozen", "closed", 33);
        assert_ne!(first, second);
        assert_eq!(
            live_rows(&[&first, &second]),
            [false, true],
            "the newer generation's publication retires the row it displaces"
        );
    });
}

#[test]
fn a_release_the_owner_could_not_admit_retires_at_the_next_release() {
    with_unmanaged_publications(|owner, publish, live_rows| {
        let row = publish("open", "frozen", 31);
        // An admission with no work left stops the release before any edit.
        let mut exhausted = owner.read_admission(0);
        assert!(owner
            .retire_released([Arc::clone(&row)], &mut exhausted)
            .is_empty());
        assert_eq!(live_rows(&[&row]), [true]);

        // The next release names nothing new and still retires it.
        let retired = owner.retire_released(std::iter::empty(), &mut owner.edit_admission());
        assert_eq!(retired, [Arc::clone(&row)]);
        assert_eq!(live_rows(&[&row]), [false]);
    });
}
