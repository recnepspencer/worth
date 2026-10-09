//! Consumed revision equality is required independently of the input digest and field prefix.
use super::*;
#[test]
fn changed_consumed_witness_with_equal_input_contacts_once_for_one_and_several_outputs() {
    let _guard = checkpoint_recovery_test_guard();
    for witnesses in [1, 2] {
        for changes in [vec![2_u64], vec![3_u64], vec![3_u64, 1]] {
            let app = installation::install_with_witnesses(witnesses);
            let (scope, principal) = installation::authentication_fixture::authenticate(&app);
            let request = app.request(&principal, &scope);
            let mut root = request
                .demand(Demand(OUTPUT))
                .start_in_program::<installation::Program, installation::Root>(&app)
                .unwrap();
            assert_eq!(settle!(root, request).0, 1);
            if witnesses == 2 {
                let mut second = request
                    .demand(Demand("anchor-c"))
                    .start_in_program::<installation::Program, installation::Root>(&app)
                    .unwrap();
                assert_eq!(settle!(second, request).0, 1);
            }
            let mut dependent = request
                .demand(counted_producer::Demand)
                .start_in_program::<installation::Program, installation::DependentRoot>(&app)
                .unwrap();
            assert_eq!(settle!(dependent, request).0, 1);
            let input = || {
                request
                    .query(counted_source::CountedRead {
                        body_key: DEPENDENT.to_owned(),
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .clone()
            };
            let before = input();
            // A changed then restored encoding has moved its revision: Fresh, one contact.
            let changed_revision = changes.iter().any(|value| value.div_ceil(2) != 1);
            for (index, value) in changes.into_iter().enumerate() {
                let observed = request
                    .query(PlanarRead {
                        body_key: "anchor-b".to_owned(),
                    })
                    .execute()
                    .unwrap();
                request
                    .mutate(PlanarSourceAdjustment {
                        scope_key: "anchor-b".to_owned(),
                        replacement_y: length(value),
                    })
                    .expect_source(observed.observed_sources()[0].clone())
                    .idempotency(&(0x61400 + index as u64))
                    .execute_performed::<installation::Program, installation::Root>(&app)
                    .unwrap();
                let mut root = request
                    .demand(Demand(OUTPUT))
                    .start_in_program::<installation::Program, installation::Root>(&app)
                    .unwrap();
                assert_eq!(
                    settle!(root, request).0,
                    1,
                    "witnesses {witnesses}, index {index}, value {value}"
                );
            }
            assert_eq!(input(), before, "the canonical source value is equal");
            counted_source::take_calls();
            dependent_publication::take_calls();
            assert_eq!(
                settle!(dependent, request).0,
                1 + usize::from(changed_revision)
            );
            assert_eq!(
                dependent_publication::take_calls(),
                usize::from(changed_revision)
            );
            assert_eq!(counted_source::take_calls(), 1);
        }
    }
}
