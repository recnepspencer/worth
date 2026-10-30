pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::request_lifetime::*;

#[test]
fn adoption_preparation_and_publication_own_separate_requests() {
    let _observation = worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let host = publish_on_first_program();
        let initial = host.current_world();
        let target = *host
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .owned_revision();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let _restore = Restore(place(placement), bound(None));
        let programs = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(initial)
            .programs();
        let requirements = programs.compare(&target).unwrap();
        reports();
        let dropped = programs
            .adopt(&requirements)
            .expect("declared adoption capability admits")
            .prepare(64)
            .unwrap();
        assert_eq!(reports().len(), 2);
        drop(dropped);
        assert!(
            reports().is_empty(),
            "dropping staged facts performs no advancement"
        );
        assert_eq!(host.current_world(), initial);
        for zero_memory in [false, true] {
            bound(None);
            let programs = host
                .runtime()
                .request(&principal, &scope)
                .on_branch(initial)
                .programs();
            let prepared = programs
                .adopt(&requirements)
                .expect("declared adoption capability admits")
                .prepare(64)
                .unwrap();
            assert_eq!(reports().len(), 2);
            bound(Some(worth_foundational::ExecutionBudget::new(
                NonZeroUsize::MIN,
                if zero_memory { Some(0) } else { None },
                Some(if zero_memory { 8_000_000 } else { 0 }),
            )));
            let before = reads();
            let Outcome::AdmissionDenied {
                cause,
                prepared: retained,
            } = prepared.publish(host.runtime(), &principal, &scope)
            else {
                panic!("publication reports its request admission refusal");
            };
            drop(retained);
            assert_eq!(reads(), before);
            if zero_memory {
                let Denial::Resource(Resource::MemoryLimit {
                    level,
                    requested,
                    admitted,
                }) = cause
                else {
                    panic!("lease bytes must be retained");
                };
                assert_eq!(level, Level::Policy);
                assert!(requested > 0);
                assert_eq!(admitted, 0);
            } else {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            }
            assert_eq!(reports().len(), 1);
            assert_eq!(host.current_world(), initial, "refusal publishes nothing");
        }
        bound(None);
        let programs = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(initial)
            .programs();
        let prepared = programs
            .adopt(&requirements)
            .expect("declared adoption capability admits")
            .prepare(64)
            .unwrap();
        assert_eq!(reports().len(), 2);
        let before = reads();
        assert!(matches!(
            prepared.publish(host.runtime(), &principal, &scope),
            Outcome::Performed(_)
        ));
        assert!(
            reads() > before,
            "admitted publication contacts its revalidation reader"
        );
        assert_eq!(
            reports().len(),
            1,
            "publish owns a fresh request after host preparation"
        );
    }
}

pub(crate) mod entry_custody;
