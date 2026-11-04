//! Installed aggregate schema and causally complete primary-graph worlds.

use std::cell::Cell;
use std::collections::BTreeMap;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::I64ApplicationValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatusBinding;
use worth_query_declaration::worth_query_application_schema;
use worth_query_declaration::worth_query_aspect;
use worth_query_declaration::worth_query_entity;
use worth_query_declaration::worth_query_field;
use worth_query_declaration::worth_query_principal_binding;
use worth_query_declaration::worth_query_relation;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;

use worth_query_execution::facade::application_invariants::WorthQueryInvariantProjectionWork;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantAggregateDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;

worth_query_entity!(pub AggregateExternalMapping for AggregateSchema);
worth_query_entity!(pub AggregatePrincipal for AggregateSchema);
worth_query_entity!(pub AggregateSource for AggregateSchema);
worth_query_entity!(pub AggregateTarget for AggregateSchema);
worth_query_aspect!(pub AggregateExternalIdentity for AggregateSchema, AggregateExternalMapping; identity = AspectIdentity(0x9161102f), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AggregateExternalIdentityField for AggregateSchema,
    AggregateExternalMapping, AggregateExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_field!(
    pub AggregateMappingStatusField for AggregateSchema,
    AggregateExternalMapping, AggregateExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_aspect!(pub AggregatePrincipalIdentity for AggregateSchema, AggregatePrincipal; identity = AspectIdentity(0x91611030), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AggregatePrincipalIdentityField for AggregateSchema,
    AggregatePrincipal, AggregatePrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_relation!(
    pub AggregateMappingTarget in AggregateSchema,
    AggregateExternalMapping => AggregatePrincipal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub AggregateIdentityBinding in AggregateSchema,
    mapping AggregateExternalMapping {
        identity: AggregateExternalIdentityField,
        status: AggregateMappingStatusField,
        target: AggregateMappingTarget => AggregatePrincipal,
        principal_identity: AggregatePrincipalIdentityField
    }
);
worth_query_aspect!(pub SourceFacts for AggregateSchema, AggregateSource; identity = AspectIdentity(0x91611031), revision = AspectContractRevision(1),);
worth_query_aspect!(pub TargetFacts for AggregateSchema, AggregateTarget; identity = AspectIdentity(0x91611032), revision = AspectContractRevision(1),);
worth_query_field!(
    pub SourceIdentity for AggregateSchema, AggregateSource, SourceFacts:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub SourceAmount for AggregateSchema, AggregateSource, SourceFacts:
    optional i64 => I64ApplicationValueBinding, read_write, no_equality
);
worth_query_field!(
    pub TargetIdentity for AggregateSchema, AggregateTarget, TargetFacts:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_relation!(
    pub AggregateContribution in AggregateSchema,
    AggregateSource => AggregateTarget; integrity = same_context_unbounded_retain_dangling);

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

pub struct AggregateWorld {
    pub _runtime: WorthQueryPrimaryGraphApplicationRuntime<AggregateSchema>,
    pub authority:
        worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority<AggregateSchema>,
    pub target: WorthQueryInvariantEntityIdentity<AggregateSchema, AggregateTarget>,
}

pub struct AggregateObservation {
    pub result: Result<(i64, u64), WorthQueryInvariantAggregateDenialKind>,
    pub work: WorthQueryInvariantProjectionWork,
}

pub struct BoundedAggregateObservation {
    pub exhausted: bool,
    pub denial: Option<WorthQueryInvariantAggregateDenialKind>,
    pub work: WorthQueryInvariantProjectionWork,
}

impl AggregateWorld {
    pub fn with_host_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
                'scope,
            >,
        ) -> R,
    ) -> Result<
        R,
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    > {
        self._runtime.with_host_advancement(body)
    }

    pub fn fixture_phase<'scope>(
        &self,
        host: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'scope>,
    ) -> worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'scope>
    {
        host.phase_for_application_for_test(&self._runtime)
    }

    pub fn values(values: impl IntoIterator<Item = i64>) -> Self {
        Self::install(values.into_iter().map(Some).collect(), false)
    }

    pub fn missing_value() -> Self {
        Self::install(vec![None], false)
    }

    pub fn ambiguous(value: i64) -> Self {
        Self::install(vec![Some(value)], true)
    }

    pub fn observe(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    ) -> AggregateObservation {
        let completed = self
            .authority
            .project(
                &self.fixture_phase(phase_custody),
                |reader| {
                    reader.summarize_exclusive_incoming(
                        AggregateContribution::reference(),
                        SourceAmount::reference(),
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

    pub fn observe_bounded(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        maximum_work: usize,
    ) -> BoundedAggregateObservation {
        let denial = Cell::new(None);
        let work = Cell::new(WorthQueryInvariantProjectionWork::default());
        let selected = self
            ._runtime
            .select_product_branch_for_test(execution, self._runtime.product_runtime().default_branch_for_test())
            .expect("aggregate product basis");
        let product = selected.product().publication_binding();
        let completed = self.authority.project_with_work_limit_for_test(
            execution,
            Some(maximum_work),
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
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        );
        BoundedAggregateObservation {
            exhausted: completed.is_err(),
            denial: denial.get(),
            work: work.get(),
        }
    }

    pub fn replace_amount(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
    pub fn replace_amount_on(
        &self,
        phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        source: worth_relational::facade::identity::EntityId,
        amount: i64,
    ) {
        let locator = (*self
            .authority
            .layout_for_test())
            .field_locator("AggregateSource", "SourceFacts", "SourceAmount")
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

    pub fn install(values: Vec<Option<i64>>, ambiguous: bool) -> Self {
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

    pub fn publish(
        runtime: WorthQueryExecutionRuntime,
        authority: worth_query_execution::facade::domain_computation::WorthQueryExecutionInstallationAuthority,
        values: Vec<Option<i64>>,
        ambiguous: bool,
    ) -> Self {
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
            let phase = &active_phase;

            let installed = runtime
                .installed_packages()
                .bind_application_schema(AggregateSchema::declaration().expect("schema redeclares"))
                .expect("aggregate schema binds");
            let mut bootstrap = authority
            .prepare_primary_graph(&phase.bootstrap_for_test(), &runtime, &installed, worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources())
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
                    &active_phase.phase_for_application_for_test(&runtime),
                    |reader| {
                        reader.resolve_entity(TargetIdentity::reference(), "target".to_owned())
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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

use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::population::bind_world;

#[path = "branches/mod.rs"]
pub mod branches;
#[path = "interrupted_refresh/mod.rs"]
pub mod interrupted_refresh;
#[path = "population/mod.rs"]
pub mod population;
