use super::*;
use primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
use worth_query_decl::facade::application_operation::ApplicationMutationBinding;

#[test]
fn configured_omissions_preserve_required_meaning_and_release_pending_installation() {
    for missing in 0..4 {
        let mut configuration = setup::configuration();
        let (kind, subject) = match missing {
            0 => {
                configuration.final_feature.initial = None;
                (Kind::MissingProducerBinding, <PlanarFinalOutputProducer<ExampleSchema> as WorthQueryApplicationProducerBinding<ExampleSchema>>::IDENTITY)
            }
            1 => {
                configuration.final_feature.preserve = None;
                (Kind::MissingProducerBinding, <PlanarFinalPreserveProducer<ExampleSchema> as WorthQueryApplicationProducerBinding<ExampleSchema>>::IDENTITY)
            }
            2 => {
                configuration
                    .final_feature
                    .initial
                    .as_mut()
                    .unwrap()
                    .handler = None;
                (
                    Kind::MissingMutationHandler,
                    <FinalPlanarMutationBinding<ExampleSchema> as ApplicationMutationBinding<
                        ExampleSchema,
                    >>::IDENTITY,
                )
            }
            _ => {
                configuration
                    .final_feature
                    .initial
                    .as_mut()
                    .unwrap()
                    .readiness = None;
                (Kind::MissingConditionalBinding, <PlanarFinalOutputReadiness<ExampleSchema> as WorthQueryApplicationConditionalBinding<ExampleSchema>>::IDENTITY)
            }
        };
        let calls = Arc::clone(&configuration.setup_calls);
        let invariant = Arc::clone(&configuration.invariant_calls);
        let probe = Arc::clone(&configuration.invariant_probe);
        let seeded = std::cell::Cell::new(false);
        let denial = match setup::install::<false>(configuration, || seeded.set(true)) {
            Err(installation::WorthQueryInMemoryApplicationDenial::Contributions(denial)) => denial,
            Err(other) => panic!("unexpected refusal: {other:?}"),
            Ok(_) => panic!("missing required configuration must not install"),
        };
        assert_eq!(denial.kind(), kind);
        assert_eq!(denial.subject(), subject);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "the real configuration callback was reached"
        );
        assert!(!seeded.get(), "no initial state or output may publish");
        assert_eq!(invariant.load(Ordering::SeqCst), 0);
        assert_eq!(
            Arc::strong_count(&invariant),
            1,
            "the previously registered invariant closure was dropped"
        );
        assert_eq!(
            Arc::strong_count(&probe),
            1,
            "pending native installation released its captured custody"
        );
    }
}

#[test]
fn omitting_both_contracts_and_registration_cannot_erase_program_requirement() {
    let configuration = setup::configuration();
    let calls = Arc::clone(&configuration.setup_calls);
    let seeded = std::cell::Cell::new(false);
    let denial = match setup::install::<true>(configuration, || seeded.set(true)) {
        Err(installation::WorthQueryInMemoryApplicationDenial::Contributions(denial)) => denial,
        Err(other) => panic!("unexpected refusal: {other:?}"),
        Ok(_) => panic!("the independent required pair must survive both omissions"),
    };
    assert_eq!(denial.kind(), Kind::MissingProducerBinding);
    assert_eq!(
        denial.subject(),
        <PlanarFinalOutputProducer<ExampleSchema> as WorthQueryApplicationProducerBinding<
            ExampleSchema,
        >>::IDENTITY
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "required contract admission precedes configure"
    );
    assert!(!seeded.get());
}
