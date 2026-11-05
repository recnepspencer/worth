use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateContribution;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateSchema;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateSource;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateTarget;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::SourceAmount;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::SourceIdentity;
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::TargetIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
pub fn bind_world(
    bootstrap: &mut worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap<
        AggregateSchema,
    >,
    values: Vec<Option<i64>>,
    ambiguous: bool,
) {
    bind_target(bootstrap, "target");
    if ambiguous {
        bind_target(bootstrap, "other-target");
    }
    for (ordinal, value) in values.into_iter().enumerate() {
        let source = format!("source-{ordinal}");
        let mut seed =
            WorthQueryApplicationEntitySeed::new(AggregateSource::reference(), entity_key(&source))
                .field(SourceIdentity::reference(), source.clone());
        if let Some(value) = value {
            seed = seed.field(SourceAmount::reference(), value);
        }
        bootstrap.bind_entity(seed).expect("source binds");
        bind_contribution(bootstrap, &source, "target", ordinal);
    }
    if ambiguous {
        bind_contribution(bootstrap, "source-0", "other-target", 99);
    }
}

pub fn bind_target(
    bootstrap: &mut worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap<
        AggregateSchema,
    >,
    target: &str,
) {
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(AggregateTarget::reference(), entity_key(target))
                .field(TargetIdentity::reference(), target.to_owned()),
        )
        .expect("target binds");
}

pub fn bind_contribution(
    bootstrap: &mut worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap<
        AggregateSchema,
    >,
    source: &str,
    target: &str,
    ordinal: usize,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            AggregateContribution::reference(),
            format!("contribution-{ordinal}"),
            entity_key(source),
            entity_key(target),
        ))
        .expect("contribution binds");
}

pub fn entity_key<Schema, Entity>(value: &str) -> WorthQueryApplicationEntityKey<Schema, Entity> {
    WorthQueryApplicationEntityKey::new(value).expect("fixture entity key is non-empty")
}
