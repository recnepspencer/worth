use super::*;
#[test]
fn cancellation_before_and_after_signal_movement_has_honest_terminal_truth() {
    for boundary in [
        Boundary::BeforeCanonicalMovement,
        Boundary::AfterCanonicalMovement,
    ] {
        let mut court = CompositeSupplyChainCourt::compile();
        let root = court.bootstrap();
        let source = RuntimeWorldCancellationSource::new();
        let token = source.token();
        let prepared = signal_prepared(&court, &root, &token);
        let mut context = court.context(&root);
        let port = court.world.publication_port();
        let signal = court
            .signal
            .owner_component_services()
            .unwrap()
            .basis_port();
        let before = signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements();
        let control = court.signal.owner_operation_control().unwrap();
        std::thread::scope(|scope| {
            let pause = control.arm_pause_once(boundary);
            let worker = scope.spawn(move || {
                port.execute_with_signal(prepared, &mut context, &token, |_| Ok(()))
            });
            assert!(pause.wait_until_reached(WAIT));
            source.cancel();
            pause.release();
            let result = worker.join().unwrap();
            let movements = signal
                .owner_service_cost_snapshot()
                .unwrap()
                .canonical_movements()
                - before;
            if boundary == Boundary::BeforeCanonicalMovement {
                assert_eq!(movements, 0);
                assert!(
                    matches!(&result,RuntimeWorldPublicationOutcome::NoEffect(no) if no.cause()==NoEffectCause::CancelledBeforeEffect)
                );
            } else {
                assert_eq!(movements, 1);
                assert!(matches!(
                    &result,
                    RuntimeWorldPublicationOutcome::ProductUnpublished(effects)
                        if effects.cause() == ProductUnpublishedCause::CancellationAfterEffect
                ));
            }
            assert_eq!(
                court.observe(&root).selected_commit(),
                root.selected_commit()
            );
            clean(&court, result);
        });
        drop(root);
        court.finish();
    }
}
#[test]
fn injected_signal_unwind_retains_any_moved_truth() {
    for boundary in [
        Boundary::BeforeCanonicalMovement,
        Boundary::AfterCanonicalMovement,
        Boundary::OutcomeConstruction,
    ] {
        let mut court = CompositeSupplyChainCourt::compile();
        let root = court.bootstrap();
        let token = RuntimeWorldCancellationSource::new().token();
        let prepared = signal_prepared(&court, &root, &token);
        let signal = court
            .signal
            .owner_component_services()
            .unwrap()
            .basis_port();
        let before = signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements();
        court
            .signal
            .owner_operation_control()
            .unwrap()
            .inject_panic_once(boundary);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            court.world.publication_port().execute_with_signal(
                prepared,
                &mut court.context(&root),
                &token,
                |_| Ok(()),
            )
        }));
        let movements = signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements()
            - before;
        if boundary == Boundary::BeforeCanonicalMovement {
            assert_eq!(movements, 0);
        } else {
            assert_eq!(movements, 1);
        }
        let payload = result.expect_err("the original Signal panic must propagate");
        assert_eq!(
            payload.downcast_ref::<String>(),
            Some(&format!(
                "injected Signal owner operation fault at {boundary:?}"
            ))
        );
        let page = court
            .world
            .inspection_port()
            .recovery_page(None, std::num::NonZeroUsize::new(4).unwrap())
            .unwrap();
        assert_eq!(page.rows().len(), usize::from(movements > 0));
        for row in page.rows() {
            let recovery = court.world.recovery_port();
            let effects = recovery.inspect_effects(row.handle()).unwrap();
            assert_eq!(effects.owner_effect_count(), 1);
            let advanced = effects
                .component_results()
                .signal()
                .advanced_outcome()
                .expect("exact Signal result survives unwind");
            assert_eq!(
                advanced.advanced_basis().observation().generation().get(),
                root.basis().signal_basis().observation().generation().get() + 1
            );
            assert_eq!(
                court.observe(&root).selected_commit(),
                root.selected_commit()
            );
            drop(effects);
            recovery.release_effects(row.handle(), 0).unwrap();
        }
        drop(root);
        court.finish();
    }
}
