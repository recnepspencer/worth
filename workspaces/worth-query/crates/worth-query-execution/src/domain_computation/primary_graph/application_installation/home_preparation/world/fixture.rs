//! Installed source query and configured provider for the attachment custody proof.
use super::*;
use worth_foundational::facade::WorthValue;
use worth_query_declaration::facade::{
    application_query::*, application_schema::*, authentication::*,
};
use worth_query_declaration::{
    worth_query_application, worth_query_application_contribution, worth_query_application_query,
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_portable_type,
    worth_query_structured_value_binding,
};

worth_query_application!(pub(super) Catalog { owner: "worth.query.home-custody", version: (1,0), contributions: [Contribution], });
worth_query_entity!(Mapping for Catalog);
worth_query_entity!(Principal for Catalog);
worth_query_aspect!(MappingFacts for Catalog, Mapping; identity = AspectIdentity(0x77190001), revision = AspectContractRevision(1));
worth_query_aspect!(Facts for Catalog, Principal; identity = AspectIdentity(0x77190002), revision = AspectContractRevision(1));
worth_query_field!(External for Catalog, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(Status for Catalog, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(Value for Catalog, Principal, Facts: u64 => U64ApplicationValueBinding, read_only, equality);
struct Target;
impl Target {
    fn reference() -> ApplicationRelationRef<Catalog, Self, Mapping, Principal> {
        ApplicationRelationRef::from_schema_identifiers(
            "Target",
            "Mapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
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
                External::reference(),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                Status::reference(),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(Target::reference()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                Value::reference(),
            ),
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
fn source() -> ApplicationQueryDefinition<Catalog, SourceQuery, (), Row, Principal> {
    ApplicationQueryDefinitionBuilder::declare(SourceQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(Principal::reference())
                .field(ApplicationQueryResultFieldRef::<
                    SourceQuery,
                    ValueSlot,
                    Catalog,
                    Principal,
                    Facts,
                    Value,
                    u64,
                    ReadOnly,
                    EqualityPredicate,
                    NoApplicationUnit,
                >::new("value", Value::reference()))
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
struct SourceBinding;
impl ApplicationQueryBinding<Catalog> for SourceBinding {
    type Input = ();
    type InputBinding = ParametersBinding;
    type Query = SourceQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = ApplicationQueryPrincipalScope<
        Catalog,
        Principal,
        Facts,
        Value,
        u64,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.home-custody.source-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(8);
    fn scope_field() -> ApplicationFieldRef<
        Catalog,
        Principal,
        Facts,
        Value,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Value::reference()
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
worth_query_application_contribution!(pub(super) contribution Contribution in Catalog {
    identity: "worth.query.home-custody.members.v1", members: |schema| {
        schema.entity(Mapping::reference()).entity(Principal::reference())
            .aspect(Mapping::reference(), MappingFacts::reference()).aspect(Principal::reference(), Facts::reference())
            .field(Mapping::reference(), External::reference()).field(Mapping::reference(), Status::reference())
            .field(Principal::reference(), Value::reference()).relation(Target::reference(), Mapping::reference(), Principal::reference())
            .principal_binding(principal_binding()).application_query(source()).application_query_binding::<SourceBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<Catalog, Index>::new())
    }
});

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
