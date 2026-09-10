use super::*;

fn assert_charge<T: Clone + RetainedStorageMeasurement, const N: usize>(
    values: &PersistentVector<T, N>,
) {
    assert_eq!(
        values.prepared_retained_charge().unwrap(),
        values
            .retained_heap_charge(&mut Work::new(usize::MAX))
            .unwrap(),
    );
}

#[test]
fn materialized_replacement_preserves_exact_charge_across_storage_shapes() {
    let mut values: PersistentVector<String, 4> =
        (0..5).map(|_| String::with_capacity(1024)).collect();
    values.prepare_retained_charge(&mut Work::new(100)).unwrap();
    let replace = |values: &mut PersistentVector<String, 4>, index| {
        let prepared = values
            .prepare_retained_replacement(index, "replacement".repeat(200), &mut Work::new(100_000))
            .unwrap();
        let required = prepared.required_charge();
        prepared.publish(required).unwrap();
        assert_charge(values);
    };
    replace(&mut values, 0); // Exclusive replacement.
    let source = values.fork_persistent();
    replace(&mut values, 1); // First base override.
    let sibling = values.clone();
    replace(&mut values, 1); // Shared existing override/page.
    values
        .push_with_retained_charge("appended".into(), &mut Work::new(100))
        .unwrap();
    let appended = values.clone();
    replace(&mut values, 5); // Shared appended value.
    for index in [2, 3, 4, 0, 1] {
        replace(&mut values, index);
    }
    assert_charge(&source);
    assert_charge(&sibling);
    assert_charge(&appended);
    assert!(source[1].is_empty());
    assert_eq!(appended[5], "appended");
}

#[test]
fn denied_replacement_returns_input_without_mutation_or_payload_clone() {
    #[derive(Debug)]
    struct Payload(String);
    impl Clone for Payload {
        fn clone(&self) -> Self {
            panic!("complete replacement must not clone the old payload")
        }
    }
    impl RetainedStorageMeasurement for Payload {
        fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
            self.0.retained_heap_charge(work)
        }
    }
    let mut source: PersistentVector<Payload> = [Payload("original".into())].into_iter().collect();
    source.prepare_retained_charge(&mut Work::new(100)).unwrap();
    let retained = source.fork_persistent();
    let mut measured = source.clone();
    let mut measure_work = Work::new(usize::MAX);
    measured
        .prepare_retained_replacement(0, Payload("next".into()), &mut measure_work)
        .unwrap();
    let cost = measure_work.visits();
    for available in [0, cost - 1, cost] {
        let mut selected = source.clone();
        let value = Payload("next".into());
        let pointer = value.0.as_ptr();
        let mut work = Work::new(cost + 7);
        work.reserve_visits(cost + 7 - available).unwrap();
        match selected.prepare_retained_replacement(0, value, &mut work) {
            Ok(prepared) => {
                let required = prepared.required_charge();
                prepared.publish(required).unwrap();
                assert_eq!(available, cost);
                assert_eq!(work.visits(), cost + 7);
                assert_eq!(selected[0].0, "next");
            }
            Err((
                value,
                RetainedVectorMutationDenial::Accounting(Denial::WorkExhausted { maximum_visits }),
            )) => {
                assert!(available < cost);
                assert_eq!(maximum_visits, cost + 7);
                assert_eq!(value.0.as_ptr(), pointer);
                assert!(selected.shares_storage_with(&source));
                assert_eq!(selected[0].0, "original");
            }
            Err((_, other)) => panic!("unexpected replacement denial: {other:?}"),
        }
        assert_charge(&selected);
    }
    assert_eq!(retained[0].0, "original");
    assert_charge(&retained);
}
