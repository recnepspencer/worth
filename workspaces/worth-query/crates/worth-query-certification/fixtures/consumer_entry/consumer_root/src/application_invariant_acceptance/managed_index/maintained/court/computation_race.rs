//! The computation's managed Parent read is the operation's recorded decision
//! fact: a Source change after the read can never commit the stale decision.
use super::super::{authoring::Edit, handler::DecisionRead, probe};
use super::*;
use std::{sync::mpsc, time::Duration};

#[derive(Clone, Copy, Debug)]
enum Hold {
    /// Inside the owner's read, at the provider's "sh" probe validation.
    InRead,
    /// After the membership read completed, at the first gather.
    AfterMembership,
}

#[test]
fn computation_index_read_change_before_commit_cannot_publish_the_stale_decision() {
    lifecycle::run(|| {
        race(Hold::InRead);
        race(Hold::AfterMembership);
    });
}

fn race(hold: Hold) {
    {
        let provider = Arc::new(prefix_observation::Calls::default());
        let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
        // The owner probes "sh"; the mutation's explicit expectation is "sha".
        let world = host::install_decision_read(
            Arc::clone(&provider),
            rules,
            DecisionRead::Partitioned(probe::Probe::Prefix("sh")),
        );
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let a = request
            .query(find("A", "sh"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        let source = request.query(find("A", "sha")).execute().unwrap();
        let expectation = source.result_set_observation().unwrap().clone();
        probe::COMPLETED.with(|runs| runs.borrow_mut().clear());
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (released_tx, released_rx) = mpsc::sync_channel(1);
        // The provider's genuine "sh" probe validation inside the computation's
        // decision read holds it while an independent writer commits a new "sh" entrant.
        let held = move || {
            entered_tx.send(()).unwrap();
            released_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("the independent writer commits while the decision is held");
        };
        match hold {
            Hold::InRead => {
                *provider.at_probe.lock().unwrap() = Some(("sh".into(), Box::new(held)))
            }
            Hold::AfterMembership => {
                probe::AFTER_MEMBERSHIP.with(|slot| *slot.borrow_mut() = Some(Box::new(held)));
            }
        }
        let world = &world;
        let result = std::thread::scope(|threads| {
            let writer = threads.spawn(move || {
                entered_rx
                    .recv_timeout(Duration::from_secs(10))
                    .expect("the computation reached the provider's refinement");
                let scope = authentication::request_scope();
                let principal = authentication::block_on(
                    authentication::admit(world.installed_schema()).authenticate(
                        authentication::LocalCredential::issued_for_model_owner(),
                        &scope,
                    ),
                )
                .unwrap();
                let changed = world
                    .runtime()
                    .request(&principal, &scope)
                    .on_branch(world.current_world())
                    .advanced()
                    .mutate(Edit::Item {
                        directory: "A".into(),
                        identity: "shy".into(),
                        name: "shy".into(),
                    })
                    .idempotency(&(0x2760_0300u64 + hold as u64))
                    .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation);
                released_tx.send(()).unwrap();
                assert!(
                    matches!(
                        changed.unwrap(),
                        WorthQueryApplicationMutationOutcome::Committed { .. }
                    ),
                    "the independent entrant commits"
                );
            });
            let result = request
                .advanced()
                .mutate(rename("sha", "shore"))
                .expect_result_set(expectation)
                .idempotency(&(0x2760_0310u64 + hold as u64))
                .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation);
            writer.join().unwrap();
            result
        });
        let runs = probe::COMPLETED.with(|runs| std::mem::take(&mut *runs.borrow_mut()));
        let refused = probe::REFUSED.with(|refused| std::mem::take(&mut *refused.borrow_mut()));
        match result {
            // Held after the read, the commit finds the owner call's recorded
            // managed Source fact stale: the only fact the entrant moved.
            Ok(WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::Stale(stale),
            )) if matches!(hold, Hold::AfterMembership) => {
                assert_eq!(stale.stale_fact_count(), 1);
                assert_eq!(runs, vec![expected("A", "shared", "sh")]);
            }
            // Held inside the read, the read itself refuses its moved head.
            Err(WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Handler(_),
            )) if matches!(hold, Hold::InRead) => {
                let [denial] = refused.as_slice() else {
                    panic!("exactly the owner's read refused: {refused:?}")
                };
                assert!(
                    matches!(
                        denial.managed_index_denial().unwrap().cause(),
                        WorthQueryManagedIndexDenialCause::Product(
                            WorthQueryProductBranchAdmissionDenial::Refused(
                                WorthQueryProductBranchAdmissionRefusalKind::ObservationStaleSourceHead
                            )
                        )
                    ),
                    "{denial:?}"
                );
            }
            other => panic!("the stale computation read must not publish: {other:?}"),
        }
        a.close();
    }
}
