use super::{
    AggregateContribution, AggregateSchema, AggregateSource, AggregateTarget, SourceAmount,
    SourceIdentity, TargetIdentity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed,
};
pub(super) fn bind_world(
    bootstrap: &mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap<
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

fn bind_target(
    bootstrap: &mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap<
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

fn bind_contribution(
    bootstrap: &mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap<
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

fn entity_key<Schema, Entity>(value: &str) -> WorthQueryApplicationEntityKey<Schema, Entity> {
    WorthQueryApplicationEntityKey::new(value).expect("fixture entity key is non-empty")
}
