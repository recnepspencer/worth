//! The original maintained catalog contribution, configuration and program.
use super::*;
pub struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("managed-catalog.maintained-contribution.v1");
    type Members = (
        CatalogRecords,
        super::super::protected::access::grants::BoxGrantRecords,
        super::super::geometry::schema::GeometryRecords,
        ContributionContracts,
        (
            correction::recorded::Note,
            correction::recorded::Mover,
            correction::recorded::Motion,
        ),
    );
}
pub struct Configuration {
    pub(super) prefix: Arc<prefix::Calls>,
    pub(super) rules: Arc<rule::Calls>,
    pub(super) decision_read: DecisionRead,
    pub(super) geometry: geometry::Configuration,
    pub(super) sources: source_authorization::Configuration,
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = Configuration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        generated::contracts(contracts)?;
        geometry::generated::contracts(contracts)
    }
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        geometry::configure(configuration.geometry, setup)?;
        source_authorization::configure(configuration.sources, setup)?;
        generated::configure(setup)?;
        geometry::generated::configure(setup)?;
        #[cfg(test)]
        let omit_refinement = configuration
            .prefix
            .omit_refinement
            .load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(test))]
        let omit_refinement = false;
        let registration = setup
            .managed_index_provider::<prefix::PrefixIndex, _>(prefix::Provider(
                configuration.prefix,
            ))
            .with_predicate_routing();
        let registration = if omit_refinement {
            registration
        } else {
            registration.with_refinement()
        };
        registration.install()?;
        correction::configure(setup)?;
        setup.handler::<mutation::Binding, _>(Handler(configuration.decision_read))?;
        setup.handler::<super::super::refinement::mutation::Binding, _>(mutation::Handler)?;
        setup.handler::<protected::expectation::Binding, _>(mutation::Handler)?;
        setup.handler::<authoring::Binding, _>(authoring::Handler)?;
        setup.handler::<protected::status::Binding, _>(protected::status::Handler)?;
        setup.invariant(
            super::super::refinement::rule::ExactName::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            super::super::refinement::rule::resolve,
        )?;
        setup.invariant(
            rule::ReservedName::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            move |resolver| rule::resolve(resolver, Arc::clone(&configuration.rules)),
        )
    }
}
pub(in super::super) struct Program;
pub(in super::super) struct Feature;
impl ApplicationFeature<Catalog> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "managed-catalog.maintained-feature.v1";
}
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        generated::program::Root,
        (
            geometry::generated::program::Root,
            generated::indexed::program::Root,
        ),
    )>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<Catalog, rule::ReservedName>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationSharedRuleRef<Catalog, geometry::rule::ProbeContact>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleList<
                ApplicationRuleAt<
                    ApplicationSharedRuleRef<Catalog, super::super::refinement::rule::ExactName>,
                    ApplicationCommitBoundary,
                >,
                ApplicationRuleLeaf,
            >,
        >,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("managed-catalog.maintained-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Catalog, Feature>()
                .provides::<generated::indexed::program::ItemsOutput>()
                .action::<generated::indexed::request::Binding, generated::indexed::program::Delivery>()
                .finish_action()
                .provides::<generated::program::DirectoryOutput>()
                .provides::<geometry::generated::program::DirectoryOutput>()
                .action::<geometry::generated::request::Binding, geometry::generated::program::Delivery>()
                .finish_action()
                .action::<generated::source::operation::Binding, generated::program::Delivery>()
                .finish_action()
                .action::<correction::note::Binding, correction::NoteDelivery>()
                .finish_action()
                .action::<correction::mover::Binding, correction::MoverDelivery>()
                .finish_action()
                .action::<correction::motion::Binding, correction::MotionDelivery>()
                .finish_action()
                .native_action::<mutation::Binding>()
                .finish_action()
                .native_action::<super::super::refinement::mutation::Binding>()
                .finish_action()
                .native_action::<protected::expectation::Binding>()
                .finish_action()
                .native_action::<source_authorization::expectation::NativeBinding>()
                .finish_action()
                .native_action::<source_authorization::expectation::CompletedBinding>()
                .finish_action()
                .native_action::<protected::status::Binding>()
                .finish_action()
                .operation::<protected::access::InspectBoxes>()
                .native_action::<authoring::Binding>()
                .finish_action()
                .native_action::<geometry::authoring::Binding>()
                .finish_action()
                .native_action::<geometry::refresh::Binding>()
                .finish_action()
                .finish(),
            generated::program::feature(),
            generated::indexed::program::feature(),
            geometry::generated::program::feature(),
        ]
    }
}

pub struct ContributionContracts;
impl worth_query_decl::facade::application_schema::fragment::Registration<Catalog>
    for ContributionContracts
{
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Catalog>, ApplicationSchemaDeclarationDenial>
    {
        Ok(
            // Declare this root's typed Source, fields, index and public bindings.
            // Its fixed contribution adds the rule and original decision reads.
            correction::declare(geometry::generated::declare(generated::declare(
                super::super::schema::declare(schema),
            )))
            .operation_managed_index_read::<query::PrefixBinding, _, _>(
                mutation::RenameItem::reference(),
            )
            .operation_read_field(
                mutation::RenameItem::reference(),
                ItemEnabled::at("ItemEnabled"),
            )
            .invariant(rule::definition()),
        )
    }
}
