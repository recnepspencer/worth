//! Installed aggregate schema and causally complete primary-graph worlds.

#[path = "world/records.rs"]
mod storage_model;
use std::cell::Cell;
use std::collections::BTreeMap;
pub use storage_model::*;

use worth_foundational::facade::AspectValue;

use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
};

use super::super::{WorthQueryInvariantAggregateDenialKind, WorthQueryInvariantEntityIdentity};
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller,
};
use crate::domain_computation::primary_graph::invariant_projection::WorthQueryInvariantProjectionWork;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphApplicationRuntime,
};

pub struct AggregateIdentityBinding;
impl AggregateIdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        AggregateSchema,
        Self,
        AggregateExternalMapping,
        AggregatePrincipal,
        <AggregatePrincipalIdentityField as RecordSelection>::Value,
        <AggregatePrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "AggregateIdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    AggregateExternalIdentityField::at("AggregateExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    AggregateMappingStatusField::at("AggregateMappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    AggregateMappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    AggregatePrincipalIdentityField::at("AggregatePrincipalIdentityField"),
                ),
            },
        )
    }
}

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct AggregateSchema;
impl Fragment for AggregateSchema {
    const NAME: &'static str = "AggregateSchema";
    type Bindings = NoBindings;
    type Members = (AggregateSchemaRecords, Contracts);
}
impl ApplicationSchema for AggregateSchema {
    const OWNER: &'static str = "aggregate_execution_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl AggregateSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<AggregateSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<AggregateSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<AggregateSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.principal_binding(AggregateIdentityBinding::reference()))
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
    pub(super) fn with_host_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(
            crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
        ) -> R,
    ) -> Result<R, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        self._runtime.with_host_advancement(body)
    }

    pub(super) fn fixture_phase<'scope>(
        &self,
        host: &crate::domain_computation::primary_graph::TestHostRequest<'scope>,
    ) -> crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope> {
        host.phase_for_application(&self._runtime)
    }

    pub(super) fn values(values: impl IntoIterator<Item = i64>) -> Self {
        Self::install(values.into_iter().map(Some).collect(), false)
    }

    pub(super) fn missing_value() -> Self {
        Self::install(vec![None], false)
    }

    pub(super) fn ambiguous(value: i64) -> Self {
        Self::install(vec![Some(value)], true)
    }

    pub(super) fn observe(
        &self,
        phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    ) -> AggregateObservation {
        let completed = self
            .authority
            .project(
                &self.fixture_phase(phase_custody),
                |reader| {
                    reader.summarize_exclusive_incoming(
                        AggregateContribution::reference(),
                        SourceAmount::at("SourceAmount"),
                        &self.target,
                    )
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("aggregate observation projection");
        let work = completed.work();
        let result = completed.output().as_ref().map_or_else(
            |denial| Err(denial.kind()),
            |aggregate| Ok((*aggregate.value(), aggregate.source_count())),
        );
        AggregateObservation { result, work }
    }

    pub(super) fn observe_bounded(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        maximum_work: usize,
    ) -> BoundedAggregateObservation {
        let denial = Cell::new(None);
        let work = Cell::new(WorthQueryInvariantProjectionWork::default());
        let selected = self
            ._runtime
            .select_product_branch(execution, self._runtime.product_runtime().default_branch())
            .expect("aggregate product basis");
        let product = selected.product().publication_binding();
        let completed = self.authority.project_with_work_limit(
            execution,
            Some(maximum_work),
            product.observation().basis().relational_basis().clone(),
            |reader| {
                let result = reader.summarize_exclusive_incoming(
                    AggregateContribution::reference(),
                    SourceAmount::at("SourceAmount"),
                    &self.target,
                );
                denial.set(result.as_ref().err().map(|error| error.kind()));
                work.set(reader.work);
                result
            },
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        );
        BoundedAggregateObservation {
            exhausted: completed.is_err(),
            denial: denial.get(),
            work: work.get(),
        }
    }

    pub(super) fn replace_amount(
        &self,
        phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        source: &str,
        amount: i64,
    ) {
        self.replace_amount_on(
            phase_custody,
            self.main_branch(),
            self.source(phase_custody, source),
            amount,
        );
    }

    /// Commits a new amount for one source already resolved, since a raw
    /// commit leaves the identity index behind the branch it moved.
    pub(super) fn replace_amount_on(
        &self,
        phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        branch: crate::basis::WorthQueryProductBranch,
        source: worth_relational::facade::identity::EntityId,
        amount: i64,
    ) {
        let locator = self
            .authority
            .layout
            .field_locator(
                "AggregateSource",
                SourceAmount::at("SourceAmount").aspect(),
                "SourceAmount",
            )
            .expect("amount field is installed")
            .clone();
        let fields =
            AspectFieldPatch::from(BTreeMap::from([(locator, AspectValue::Int64(amount))]));
        let intent = MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: source,
                fields,
            },
        ));
        self.commit_on(phase_custody, branch, "aggregate-stale-generation", intent);
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
        crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
            let phase = &active_phase;

            let installed = runtime
                .installed_packages()
                .bind_application_schema(AggregateSchema::declaration().expect("schema redeclares"))
                .expect("aggregate schema binds");
            let mut bootstrap = authority
            .prepare_primary_graph(&phase.bootstrap_for_test(), &runtime, &installed, crate::domain_computation::execution_runtime::product_world::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()))
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
            let budget =
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development();
            let runtime = bootstrap
                .publish_application_runtime(
                    &phase.bootstrap_for_test(),
                    runtime,
                    authority,
                    installed,
                    budget,
                )
                .expect("primary graph publishes");
            let target = projection
                .project(
                    &active_phase.phase_for_application(&runtime),
                    |reader| {
                        reader.resolve_entity(
                            TargetIdentity::at("TargetIdentity"),
                            "target".to_owned(),
                        )
                    },
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
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
        })
    }
}

#[path = "world/branches.rs"]
mod branches;
#[path = "world/population.rs"]
mod population;
use population::bind_world;
#[path = "world/interrupted_refresh.rs"]
mod interrupted_refresh;
