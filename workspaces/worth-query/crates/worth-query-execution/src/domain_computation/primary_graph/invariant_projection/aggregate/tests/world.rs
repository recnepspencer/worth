//! Installed aggregate schema and causally complete primary-graph worlds.

use std::cell::Cell;
use std::collections::BTreeMap;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::{WorthQueryInvariantAggregateDenialKind, WorthQueryInvariantEntityIdentity};
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller,
};
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryInvariantProjectionWork;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphApplicationRuntime,
};

worth_query_entity!(pub AggregateExternalMapping in AggregateSchema);
worth_query_entity!(pub AggregatePrincipal in AggregateSchema);
worth_query_entity!(pub AggregateSource in AggregateSchema);
worth_query_entity!(pub AggregateTarget in AggregateSchema);
worth_query_aspect!(pub AggregateExternalIdentity in AggregateSchema, AggregateExternalMapping; identity = AspectIdentity(0x9161102f), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AggregateExternalIdentityField in AggregateSchema,
    AggregateExternalMapping, AggregateExternalIdentity:
    WorthQueryExternalPrincipalIdentity, read_only, equality
);
worth_query_field!(
    pub AggregateMappingStatusField in AggregateSchema,
    AggregateExternalMapping, AggregateExternalIdentity:
    WorthQueryPrincipalMappingStatus, read_write, equality
);
worth_query_aspect!(pub AggregatePrincipalIdentity in AggregateSchema, AggregatePrincipal; identity = AspectIdentity(0x91611030), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AggregatePrincipalIdentityField in AggregateSchema,
    AggregatePrincipal, AggregatePrincipalIdentity:
    u64, read_only, equality
);
worth_query_relation!(
    pub AggregateMappingTarget in AggregateSchema,
    AggregateExternalMapping => AggregatePrincipal
);
worth_query_principal_binding!(
    pub AggregateIdentityBinding in AggregateSchema,
    mapping AggregateExternalMapping {
        identity: AggregateExternalIdentityField,
        status: AggregateMappingStatusField,
        target: AggregateMappingTarget => AggregatePrincipal,
        principal_identity: AggregatePrincipalIdentityField
    }
);
worth_query_aspect!(pub SourceFacts in AggregateSchema, AggregateSource; identity = AspectIdentity(0x91611031), revision = AspectContractRevision(1),);
worth_query_aspect!(pub TargetFacts in AggregateSchema, AggregateTarget; identity = AspectIdentity(0x91611032), revision = AspectContractRevision(1),);
worth_query_field!(
    pub SourceIdentity in AggregateSchema, AggregateSource, SourceFacts:
    String, read_only, equality
);
worth_query_field!(
    pub SourceAmount in AggregateSchema, AggregateSource, SourceFacts:
    optional i64, read_write, no_equality
);
worth_query_field!(
    pub TargetIdentity in AggregateSchema, AggregateTarget, TargetFacts:
    String, read_only, equality
);
worth_query_relation!(
    pub AggregateContribution in AggregateSchema,
    AggregateSource => AggregateTarget
);

worth_query_application_schema! {
    pub schema AggregateSchema {
        owner: aggregate_execution_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(AggregateExternalMapping::reference())
                .entity(AggregatePrincipal::reference())
                .entity(AggregateSource::reference())
                .entity(AggregateTarget::reference())
                .aspect(
                    AggregateExternalMapping::reference(),
                    AggregateExternalIdentity::reference(),
                )
                .aspect(
                    AggregatePrincipal::reference(),
                    AggregatePrincipalIdentity::reference(),
                )
                .field(
                    AggregateExternalMapping::reference(),
                    AggregateExternalIdentityField::reference(),
                )
                .field(
                    AggregateExternalMapping::reference(),
                    AggregateMappingStatusField::reference(),
                )
                .field(
                    AggregatePrincipal::reference(),
                    AggregatePrincipalIdentityField::reference(),
                )
                .aspect(AggregateSource::reference(), SourceFacts::reference())
                .aspect(AggregateTarget::reference(), TargetFacts::reference())
                .field(AggregateSource::reference(), SourceIdentity::reference())
                .field(AggregateSource::reference(), SourceAmount::reference())
                .field(AggregateTarget::reference(), TargetIdentity::reference())
                .relation(
                    AggregateMappingTarget::reference(),
                    AggregateExternalMapping::reference(),
                    AggregatePrincipal::reference(),
                )
                .relation(
                    AggregateContribution::reference(),
                    AggregateSource::reference(),
                    AggregateTarget::reference(),
                )
                .principal_binding(AggregateIdentityBinding::reference())
        }
    }
}

pub(super) struct AggregateWorld {
    _runtime: WorthQueryPrimaryGraphApplicationRuntime<AggregateSchema>,
    authority:
        super::super::super::WorthQueryApplicationInvariantProjectionAuthority<AggregateSchema>,
    target: WorthQueryInvariantEntityIdentity<AggregateSchema, AggregateTarget>,
}

pub(super) struct AggregateObservation {
    pub(super) result: Result<(i64, u64), WorthQueryInvariantAggregateDenialKind>,
    pub(super) work: WorthQueryInvariantProjectionWork,
}

pub(super) struct BoundedAggregateObservation {
    pub(super) exhausted: bool,
    pub(super) denial: Option<WorthQueryInvariantAggregateDenialKind>,
    pub(super) work: WorthQueryInvariantProjectionWork,
}

impl AggregateWorld {
    pub(super) fn values(values: impl IntoIterator<Item = i64>) -> Self {
        Self::install(values.into_iter().map(Some).collect(), false)
    }

    pub(super) fn missing_value() -> Self {
        Self::install(vec![None], false)
    }

    pub(super) fn ambiguous(value: i64) -> Self {
        Self::install(vec![Some(value)], true)
    }

    pub(super) fn observe(&self) -> AggregateObservation {
        let completed = self
            .authority
            .project(|reader| {
                reader.summarize_exclusive_incoming(
                    AggregateContribution::reference(),
                    SourceAmount::reference(),
                    &self.target,
                )
            })
            .expect("aggregate observation projection");
        let work = completed.work();
        let result = completed.output().as_ref().map_or_else(
            |denial| Err(denial.kind()),
            |aggregate| Ok((*aggregate.value(), aggregate.source_count())),
        );
        AggregateObservation { result, work }
    }

    pub(super) fn observe_bounded(&self, maximum_work: usize) -> BoundedAggregateObservation {
        let denial = Cell::new(None);
        let work = Cell::new(WorthQueryInvariantProjectionWork::default());
        let product = self
            ._runtime
            .admit_product_publication()
            .expect("aggregate product basis");
        let completed = self.authority.project_bounded(
            maximum_work,
            product.observation().basis().relational_basis().clone(),
            |reader| {
                let result = reader.summarize_exclusive_incoming(
                    AggregateContribution::reference(),
                    SourceAmount::reference(),
                    &self.target,
                );
                denial.set(result.as_ref().err().map(|error| error.kind()));
                work.set(reader.work);
                result
            },
        );
        BoundedAggregateObservation {
            exhausted: completed.is_err(),
            denial: denial.get(),
            work: work.get(),
        }
    }

    pub(super) fn replace_amount(&self, source: &str, amount: i64) {
        let source = self
            .authority
            .project(|reader| reader.resolve_entity(SourceIdentity::reference(), source.to_owned()))
            .expect("source projection")
            .output()
            .as_ref()
            .expect("source identity resolves")
            .clone();
        let locator = self
            .authority
            .layout
            .field_locator("AggregateSource", "SourceFacts", "SourceAmount")
            .expect("amount field is installed")
            .clone();
        self.authority.graph.with_runtime_mut(|runtime| {
            let fields =
                AspectFieldPatch::from(BTreeMap::from([(locator, AspectValue::Int64(amount))]));
            let intent = MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: source.entity_id,
                    fields,
                },
            ));
            let mut transaction = {
                let transaction_validation_input = runtime
                    .admit_branch_basis(&runtime.main_branch_identity())
                    .expect("main branch binding");
                runtime
                    .begin_branch_transaction(
                        &transaction_validation_input,
                        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .expect("owner-admitted transaction context")
            };
            transaction
                .push_batch(WorkerIntentBatch::new("aggregate-stale-generation").push(intent))
                .expect("test staging stays within configured resource budgets");
            let committed = transaction
                .commit(runtime)
                .expect("amount replacement commits");
            crate::relational_snapshot_release::release_query_snapshot(
                runtime,
                &committed.snapshot,
            );
        });
    }

    fn install(values: Vec<Option<i64>>, ambiguous: bool) -> Self {
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "aggregate_execution_test",
            1,
            0,
        ))
        .application_schema(AggregateSchema::declaration().expect("aggregate schema declares"))
        .validate()
        .expect("aggregate package validates");
        let admitted = WorthQueryInstallationAdmissionProfile::new(
            "aggregate_execution_test",
            "aggregate-owner-proof",
        )
        .admit(package)
        .expect("aggregate package admits");
        let installation = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .expect("aggregate package installs");
        let (runtime, authority) = installation.into_parts();
        Self::publish(runtime, authority, values, ambiguous)
    }

    fn publish(
        runtime: WorthQueryExecutionRuntime,
        authority: crate::domain_computation::execution_runtime::WorthQueryExecutionInstallationAuthority,
        values: Vec<Option<i64>>,
        ambiguous: bool,
    ) -> Self {
        let installed = runtime
            .installed_packages()
            .bind_application_schema(AggregateSchema::declaration().expect("schema redeclares"))
            .expect("aggregate schema binds");
        let mut bootstrap = authority
            .prepare_primary_graph(&runtime, &installed)
            .expect("primary graph prepares");
        let binding = installed
            .principal_binding(AggregateIdentityBinding::reference())
            .expect("aggregate principal binding is installed");
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("aggregate-principal")
                    .expect("principal key is valid"),
                1,
                WorthQueryExternalPrincipalIdentity::new(
                    "https://aggregate.test",
                    "aggregate-principal",
                )
                .expect("external principal identity is valid"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .expect("aggregate principal binds");
        bind_world(&mut bootstrap, values, ambiguous);
        let projection = bootstrap.retain_invariant_projection_authority();
        let runtime = bootstrap
            .publish_application_runtime(runtime, authority, installed)
            .expect("primary graph publishes");
        let target = projection
            .project(|reader| {
                reader.resolve_entity(TargetIdentity::reference(), "target".to_owned())
            })
            .expect("target projection")
            .output()
            .as_ref()
            .expect("target resolves")
            .clone();
        Self {
            _runtime: runtime,
            authority: projection,
            target,
        }
    }
}

fn bind_world(
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
