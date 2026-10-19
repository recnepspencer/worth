//! Real MODE2 installation; its handler/candidate code is shared with topology_entry.
use super::fixture_values::PositiveLength;
use super::*;
use crate::facade::application_installation::{
    in_memory_program, WorthQueryInMemoryApplicationLimits, WorthQueryProgramApplicationRuntime,
};
use crate::facade::runtime::WorthQueryApplicationCandidateResourceProfile;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding, application_program::*,
    application_schema::*,
};

worth_query_decl::facade::worth_query_application! {
    pub(super) OwnerSchema { owner: "worth.query.test.mixed-owner", version: (1, 0), contributions: [OwnerMembers], }
}
impl TopologySchemaBinding for OwnerSchema {}
worth_query_decl::facade::worth_query_application_contribution! {
    pub(super) contribution OwnerMembers in OwnerSchema {
        identity: "worth.query.test.mixed-owner-members",
        members: |schema| {
            let schema = base_model::declare(schema);
            let schema = facts::declare(schema);
            region_output::declare(schema)
                .application_query(planar_query_definition::<OwnerSchema>())
                .application_query_binding::<PlanarReadBinding<OwnerSchema>>()
        }
    }
}
impl WorthQueryApplicationContribution<OwnerSchema> for OwnerMembers {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<OwnerSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<RegionOutputProducer<OwnerSchema, false, 2>>()?;
        contracts.conditional::<RegionOutputReadiness<OwnerSchema, false, 2>>()?;
        Ok(())
    }
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, OwnerSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            PositivePlanarTurn::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            |resolver| {
                planar_invariant::resolve_rule(
                    resolver,
                    Arc::new(AtomicUsize::new(0)),
                    Arc::new(AtomicUsize::new(0)),
                )
            },
        )?;
        let handler = RegionOutputHandler::running(|reader, set| {
            WorthQueryDecisionPlan::immediate(
                match reader.field(set, facts::EntrySetWeight::reference()) {
                    Ok(value) => HandlerResult::Completed(value),
                    Err(cause) => HandlerResult::ExecutionDenied(cause),
                },
            )
        })
        .with_generated_payload();
        setup.handler::<RegionOutputBinding<OwnerSchema>, _>(handler)?;
        setup.producer::<RegionOutputProducer<OwnerSchema, false, 2>>(RegionOutputProvider::<2>)?;
        setup.conditional::<RegionOutputReadiness<OwnerSchema, false, 2>>(())?;
        Ok(())
    }
}
struct OwnerFeature;
impl ApplicationFeature<OwnerSchema> for OwnerFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.mixed-owner-feature";
}
pub(super) struct OwnerProgram;
impl ApplicationProgramDefinition<OwnerSchema> for OwnerProgram {
    type Contributions = (OwnerMembers,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<OwnerSchema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.mixed-owner-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<OwnerSchema, OwnerFeature>()
            .conditional_operation::<TotalRegionOutput>()
            .finish()]
    }
}
pub(super) type Application = WorthQueryProgramApplicationRuntime<OwnerSchema, OwnerProgram>;
pub(super) const ANCHOR: &str = "owner-source:a";
pub(super) fn install() -> Application {
    let defaults = WorthQueryApplicationCandidateResourceProfile::default();
    let required_bytes = u64::try_from(
        <RegionOutputBinding<OwnerSchema> as ApplicationMutationBinding<OwnerSchema>>::CANDIDATES
            .fixed_requirements()
            .expect("fixed candidate contract")
            .resources()
            .maximum_retained_representation_bytes(),
    )
    .expect("the real producer candidate requirement fits the host resource type");
    let candidates = WorthQueryApplicationCandidateResourceProfile::physical_resources(
        defaults.maximum_items(),
        defaults
            .maximum_retained_representation_bytes()
            .max(required_bytes),
    )
    .expect("the default host capacity and declared producer requirement are nonzero");
    in_memory_program(
        ApplicationProgramAuthoring::<OwnerSchema, OwnerProgram>::begin().validated_program().unwrap(),
        OwnerSchema::declaration().unwrap(), ((),),
        WorthQueryInMemoryApplicationLimits::new(
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
            candidates, Default::default(),
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development()),
        |_, graph, installed| {
            let binding = installed.principal_binding(ConsumerPrincipalBinding::reference()).unwrap();
            graph.bind_principal(&binding, WorthQueryApplicationPrincipalKey::new("owner").unwrap(), 1_u64,
                primary_graph::tests::fixture::external_identity("alice"),
                worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            for (key, x, y) in [(ANCHOR, 1, 1), ("owner-source:b", 10, 1), ("owner-source:c", 1, 10)] {
                graph.bind_entity(WorthQueryApplicationEntitySeed::new(Body::reference(), WorthQueryApplicationEntityKey::new(key).unwrap())
                    .field(BodyKey::reference(), key.to_owned())
                    .field(Length::reference(), PositiveLength::new(11).unwrap())
                    .field(PositionX::reference(), PositiveLength::new(x).unwrap())
                    .field(PositionY::reference(), PositiveLength::new(y).unwrap()))?;
            }
            for (from, to) in [(ANCHOR, "owner-source:b"), ("owner-source:b", "owner-source:c"), ("owner-source:c", ANCHOR)] {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(PlanarSuccessor::reference(), format!("{from}-to-{to}"),
                    WorthQueryApplicationEntityKey::new(from).unwrap(), WorthQueryApplicationEntityKey::new(to).unwrap()))?;
            }
            graph.bind_entity(WorthQueryApplicationEntitySeed::new(facts::EntrySet::reference(), WorthQueryApplicationEntityKey::new("odd").unwrap())
                .field(facts::EntrySetKey::reference(), "odd".to_owned()).field(facts::EntrySetWeight::reference(), 7))?;
            Ok(())
        }).unwrap()
}
