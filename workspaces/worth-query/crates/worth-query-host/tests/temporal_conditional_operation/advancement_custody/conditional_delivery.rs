//! The public phase-taking delivery door borrows its own installed opener.
use super::*;
#[test]
fn conditional_delivery_refuses_before_its_retained_truth_reader() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let world = CourtroomWorld::publish("blocked");
        let branch = world.application.current_world();
        let mut publication = world
            .change_input_on_branch(branch, "ready")
            .require_committed()
            .unwrap();
        let mut change = publication.take_performed_relational_product_change();
        let selected = world.application.on_branch(branch).select().unwrap();
        let scope = super::super::world::request_scope();
        for memory in [false, true] {
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if memory { 0 } else { 64 * 1024 * 1024 },
                if memory { 8_000_000 } else { 0 },
            )));
            let before = reads();
            let denied = world
                .application
                .with_application_advancement(&scope, |phase| {
                    selected.deliver_relational_change_to_conditional(
                        &phase,
                        &world.clock,
                        0,
                        change.take().unwrap(),
                    )
                })
                .unwrap_err();
            if !memory {
                assert_eq!(denied, Denial::Resource(Resource::WorkExhausted));
            } else if placement == Placement::Serial {
                assert_eq!(denied, Denial::Resource(Resource::PolicyMemoryLimit));
            } else {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = denied
                else {
                    panic!("exact bytes")
                };
                assert_eq!(level, worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            }
            assert_eq!(reads(), before);
            assert!(
                change.is_some(),
                "an opening refusal never enters the delivery reader"
            );
        }
        bound(None);
        let before = reads();
        let outcome = world
            .application
            .with_application_advancement(&scope, |phase| {
                selected.deliver_relational_change_to_conditional(
                    &phase,
                    &world.clock,
                    0,
                    change.take().unwrap(),
                )
            })
            .unwrap()
            .unwrap();
        let worth_query_host::facade::runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) = outcome else { panic!("the real conditional sink accepts the patch") };
        assert_eq!(delivery.source_envelopes_loaded(), 1);
        assert!(
            reads() > before,
            "admitted delivery contacts the same retained-truth reader"
        );
    }
}

#[test]
fn foreign_phase_cannot_deliver_on_another_installed_world() {
    use worth_query_host::facade::runtime::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind as Kind;
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let a = CourtroomWorld::publish("blocked");
        let b = CourtroomWorld::publish("blocked");
        let mut a_publication = a
            .change_input_on_branch(a.application.current_world(), "ready")
            .require_committed()
            .unwrap();
        let mut b_publication = b
            .change_input_on_branch(b.application.current_world(), "ready")
            .require_committed()
            .unwrap();
        let a_selected = a
            .application
            .on_branch(a.application.current_world())
            .select()
            .unwrap();
        let b_selected = b
            .application
            .on_branch(b.application.current_world())
            .select()
            .unwrap();
        let scope = super::super::world::request_scope();
        reports();
        // First isolate the refused work: neither installation charges anything for B.
        a.application
            .with_application_advancement(&scope, |phase| {
                let before = reads();
                let denied = b_selected
                    .deliver_relational_change_to_conditional(
                        &phase,
                        &b.clock,
                        0,
                        b_publication
                            .take_performed_relational_product_change()
                            .unwrap(),
                    )
                    .unwrap_err();
                assert_eq!(denied.kind(), Kind::ExecutionRequest(Denial::ForeignPhase));
                assert_eq!(reads(), before);
            })
            .unwrap();
        let refused = reports();
        assert_eq!(refused.len(), 1, "B never opens a request");
        assert_eq!(refused[0].as_ref().unwrap().charged_work(), 0);
        let before = reads();
        a.application.with_application_advancement(&scope, |phase| {
            let admitted = a_selected.deliver_relational_change_to_conditional(
                &phase, &a.clock, 0,
                a_publication.take_performed_relational_product_change().unwrap(),
            ).unwrap();
            assert!(matches!(admitted,
                worth_query_host::facade::runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(_)));
        }).unwrap();
        assert!(
            reads() > before,
            "the same delivery on A reaches its reader"
        );
    }
}
