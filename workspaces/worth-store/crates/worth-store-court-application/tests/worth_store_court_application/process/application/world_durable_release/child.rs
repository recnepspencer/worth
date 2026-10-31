//! Kill only after the actual Native publication reaches physical durability.
use super::*;

#[test]
fn original_world_attempt_child() {
    let Some(location) = std::env::var_os(LOCATION) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(std::path::Path::new(&location), &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let cancellation = RuntimeWorldCancellationSource::new();
    std::fs::write(markers.join("handler-runs"), "1").unwrap();
    let prepared = fixture.prepare_publication(
        &execution,
        bootstrap.product_branch().clone(),
        crate::batches::create(0),
        &cancellation.token(),
    );
    // These are the actual issued values, never inferred from equal counters.
    let original = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{:?}\n{:?}\n",
        prepared.attempt_identity().ordinal(),
        prepared.reserved_commit_identity().ordinal(),
        prepared.unpublished_recovery_handle().identity().ordinal(),
        bootstrap.product_branch().selected_commit().ordinal(),
        bootstrap.product_branch().lifecycle_incarnation().ordinal(),
        bootstrap.product_branch().branch_identity().name().as_str(),
        fixture.home.home_identity(),
        fixture.home.incarnation(),
    );
    std::fs::write(markers.join("original"), original).unwrap();
    // Bootstrap, durable Open and the one Native preparation have completed.
    // There is no further World publication before Native canonical publish.
    let gate = fixture.home.physical_serving_for_test()
        .certification_pause_physical_mutation_at(
            worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement
        );
    let publication = fixture.home.world().publication_port();
    let token = cancellation.token();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let request = crate::workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            let mut outcome =
                publication.execute_durable_without_signal(execution, prepared, &token);
            loop {
                match outcome {
                    WorldDurablePublicationOutcome::Performed(performed) => {
                        drop(performed.consume());
                        return Ok(());
                    }
                    WorldDurablePublicationOutcome::Pending(pending) => {
                        outcome = pending.advance(&execution);
                    }
                    other => {
                        return Err(format!(
                            "actual Native/World publication refused: {other:?}"
                        ))
                    }
                }
            }
        });
        assert!(crate::process_kill::await_mutation_seam(
            &gate,
            &worker,
            std::time::Instant::now() + crate::killed_child::PARK_DEADLINE
        ));
        let inspection = fixture.home.world().inspection_port();
        assert_eq!(
            inspection.history_snapshot().unwrap().installed_commits(),
            1
        );
        assert!(inspection
            .performed_publication_page(None, maximum())
            .unwrap()
            .rows()
            .is_empty());
        assert_eq!(bootstrap.product_branch().reference_generation().get(), 0);
        crate::killed_child::announce(
            &markers.join("ready"),
            b"Native durable; World occurrence absent",
        );
        crate::killed_child::park();
    });
}
