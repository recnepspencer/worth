use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};

use super::*;

pub(crate) fn complete_dependency_aba_advances_the_live_demand(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start()
        .expect("the output demand starts");
    macro_rules! settle {
        ($stage:literal) => {
            (0..32)
                .find_map(
                    |_| match demand.advance(&request).expect("the live demand advances") {
                        WorthQueryApplicationOutputDemandProgress::Pending => None,
                        WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                    },
                )
                .unwrap_or_else(|| {
                    panic!(
                        "the live demand settles within bounded progress: {}",
                        $stage
                    )
                })
        };
    }

    let first_a = settle!("first A");
    let first_a_commit = first_a
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .clone();
    drop(first_a);
    let retry_a = settle!("unchanged first A");
    assert_eq!(
        retry_a
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &first_a_commit,
        "an unchanged dependency must reuse the current producer receipt"
    );
    drop(retry_a);

    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the direct source remains readable");
    let changed = request
        .mutate(PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-a".to_owned(),
                value: length(7),
            }),
            validator_work: 4_096,
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&10_052)
        .execute()
        .expect("the producer dependency advances to B");
    let b_commit = changed
        .receipt()
        .expect("the dependency change commits")
        .committed_product_publication()
        .composite_commit()
        .clone();

    let second_a = settle!("A after B");
    let second_a_commit = second_a
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .clone();
    assert_ne!(
        second_a_commit, first_a_commit,
        "A after B must not replay A"
    );
    assert_ne!(
        second_a_commit, b_commit,
        "A after B must publish its own receipt"
    );
    assert_eq!(
        request
            .at(second_a.observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the reconstructed A output is readable")
            .rows()[0]
            .value,
        length(2)
    );
    drop(second_a);

    let retry_second_a = settle!("unchanged second A");
    assert_eq!(
        retry_second_a
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &second_a_commit,
        "the converged A state must reuse its newest receipt"
    );
    drop(retry_second_a);
    demand.close();
}
