//! Installed source query and configured provider for the attachment custody proof.
use super::*;
use worth_foundational::facade::WorthValue;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
use worth_query_declaration::facade::{
    application_query::*, application_schema::*, authentication::*,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};
#[path = "fixture/records.rs"]
mod records;
use records::*;

pub(super) struct Catalog;
impl Fragment for Catalog {
    const NAME: &'static str = "Catalog";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Catalog {
    const OWNER: &'static str = "worth.query.home-custody";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for Catalog {
    type Contributions = (Contribution,);
}
struct PrincipalBinding;
fn principal_binding() -> ApplicationPrincipalBindingRef<
    Catalog,
    PrincipalBinding,
    Mapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                External::at("External"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(Status::at(
                "Status",
            )),
            target: ApplicationPrincipalTargetRequirement::from_relation(Target::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(Value::at(
                "Value",
            )),
        },
    )
}
#[derive(WorthValue)]
struct Row {
    value: u64,
}
worth_query_structured_value_binding!(RowBinding for Row { identity: "worth.query.home-custody.row.v1" });
worth_query_structured_value_binding!(ParametersBinding for () { identity: "worth.rust.unit" });
worth_query_application_query!(SourceQuery for Catalog, identity "worth.query.home-custody.source.v1", parameters ParametersBinding,
    result RowBinding, scope Principal => "Principal", name "source");
struct ValueSlot;
worth_query_portable_type!(ValueSlot => "worth.query.home-custody.value.v1");
fn source() -> ApplicationQueryDefinitionBuilder<Catalog, SourceQuery, (), Row, Principal> {
    ApplicationQueryDefinitionBuilder::declare(SourceQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Catalog, SourceQuery, Principal, Row, RowBinding>::new(Principal::reference())
                .field(ApplicationQueryResultFieldRef::<SourceQuery, ValueSlot, Catalog, Value>::new("value", Value::at("Value")))
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
struct SourceBinding;
impl ApplicationQueryBinding<Catalog> for SourceBinding {
    type Input = ();
    type InputBinding = ParametersBinding;
    type Query = SourceQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = ApplicationQueryPrincipalScope<Value>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.home-custody.source-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
    fn scope_field() -> Value {
        Value::at("Value")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Catalog,
        PrincipalBinding,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        principal_binding()
    }
}
struct Index;
#[derive(WorthValue)]
struct Probe {
    minimum: u64,
}
worth_query_declaration::worth_query_value_binding!(ProbeBinding for Probe {
    identity: "worth.query.home-custody.probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});
impl ApplicationManagedIndexBinding<Catalog> for Index {
    type SourceBinding = SourceBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = Principal;
    const NAME: &'static str = "custody_index";
    const PROVIDER: &'static str = "worth.query.home-custody.provider.v1";
    const PROVIDER_VERSION: u32 = 1;
}
pub(super) struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.home-custody.members.v1");
    type Members = (IdentityRecords, SourceContracts);
}
pub(super) struct SourceContracts;
impl Registration<Catalog> for SourceContracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Catalog>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .principal_binding(principal_binding())
            .application_query(source())
            .application_query_binding::<SourceBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<Catalog, Index>::new()))
    }
}

pub(super) struct Program;
struct Feature;
impl ApplicationFeature<Catalog> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.home-custody.feature.v1";
}
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home-custody.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Catalog, Feature>().finish()]
    }
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = Provider;
    fn configure(
        provider: Provider,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.managed_index::<Index, _>(provider)
    }
}
pub(super) struct Provider(pub(super) Arc<AtomicUsize>);
impl Drop for Provider {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl WorthQueryManagedIndexProvider<Catalog, Index> for Provider {
    type Node = u64;
    type Summary = ();
    type Denial = std::convert::Infallible;
    const IDENTITY: &'static str = Index::PROVIDER;
    const VERSION: u32 = Index::PROVIDER_VERSION;
    fn node_capacity(
        &self,
        n: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(n)
    }
    fn build(
        &self,
        _: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
        _: &mut WorthQueryManagedIndexNodeWriter<'_, u64>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<
        Option<WorthQueryManagedIndexNodeId>,
        WorthQueryManagedIndexProviderStop<Self::Denial>,
    > {
        control.checkpoint(1)?;
        Ok(None)
    }
    fn query(
        &self,
        _: &WorthQueryManagedIndexSourceRows<'_, Catalog, Index>,
        _: &WorthQueryManagedIndexNodes<u64>,
        _: &Probe,
        _: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(())
    }
}

pub(super) fn seed_principal(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
    installed: &WorthQueryInstalledApplicationSchema<Catalog>,
) {
    bootstrap
        .bind_principal(
            &installed.principal_binding(principal_binding()).unwrap(),
            WorthQueryApplicationPrincipalKey::new("custody-reader").unwrap(),
            1u64,
            WorthQueryExternalPrincipalIdentity::new("home-custody", "reader").unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
}
