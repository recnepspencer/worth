mod producer_contract_denials;
mod producer_denials;
mod program;

use program::{validated_denial_program, DenialProgram};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity, ApplicationSchema,
    ApplicationSchemaComposition, ApplicationSchemaContribution,
    ApplicationSchemaContributionIdentity, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    application_installation,
    application_installation::{
        WorthQueryApplicationLimits, WorthQueryApplicationPreparationDenial,
    },
    primary_graph::{
        self, WorthQueryPrimaryGraphInstallationDenial,
        WorthQueryPrimaryGraphInstallationDenialKind,
    },
    runtime,
};
use worth_query_parameter_entry::{ParameterContribution, ParameterSchemaBinding};
use worth_query_topology_entry::{
    PlanarHandler, PlanarMutationBinding, PositivePlanarTurn, PositiveTurnRule,
    TopologyConfiguration, TopologyContribution, TopologySchemaBinding,
};

pub(super) fn run() {
    inventory_drift_precedes_callbacks();
    missing_handler_precedes_initializer();
    foreign_member_registration_is_denied();
    producer_denials::run();
    producer_contract_denials::run();
}

// The same real entry declarations are retained while each fixture varies only
// the contribution boundary under attack.

struct MissingContributionSchema;
impl TopologySchemaBinding for MissingContributionSchema {
    type PrincipalFragment = worth_query_topology_entry::PrincipalRecords;
}
impl ParameterSchemaBinding for MissingContributionSchema {}
impl worth_query_decl::facade::application_schema::fragment::Fragment
    for MissingContributionSchema
{
    const NAME: &'static str = "MissingContributionSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (TopologyContribution, ParameterContribution);
}
impl ApplicationSchema for MissingContributionSchema {
    const OWNER: &'static str = "worth.query.certification.contribution-denials";
    const NAME: &'static str = "MissingContributionSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for MissingContributionSchema {
    type Contributions = (TopologyContribution,);
}

struct DuplicateContributionSchema;
impl TopologySchemaBinding for DuplicateContributionSchema {
    type PrincipalFragment = worth_query_topology_entry::PrincipalRecords;
}
impl ParameterSchemaBinding for DuplicateContributionSchema {}
impl worth_query_decl::facade::application_schema::fragment::Fragment
    for DuplicateContributionSchema
{
    const NAME: &'static str = "DuplicateContributionSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (TopologyContribution, ParameterContribution);
}
impl ApplicationSchema for DuplicateContributionSchema {
    const OWNER: &'static str = "worth.query.certification.contribution-denials";
    const NAME: &'static str = "DuplicateContributionSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for DuplicateContributionSchema {
    type Contributions = (
        TopologyContribution,
        ParameterContribution,
        ParameterContribution,
    );
}

struct MissingHandlerSchema;
impl TopologySchemaBinding for MissingHandlerSchema {
    type PrincipalFragment = worth_query_topology_entry::PrincipalRecords;
}
impl ParameterSchemaBinding for MissingHandlerSchema {}
impl worth_query_decl::facade::application_schema::fragment::Fragment for MissingHandlerSchema {
    const NAME: &'static str = "MissingHandlerSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (HandlerOmissionContribution, ParameterContribution);
}
impl ApplicationSchema for MissingHandlerSchema {
    const OWNER: &'static str = "worth.query.certification.contribution-denials";
    const NAME: &'static str = "MissingHandlerSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for MissingHandlerSchema {
    type Contributions = (HandlerOmissionContribution, ParameterContribution);
}

struct ForeignMemberSchema;
impl TopologySchemaBinding for ForeignMemberSchema {
    type PrincipalFragment = worth_query_topology_entry::PrincipalRecords;
}
impl ParameterSchemaBinding for ForeignMemberSchema {}
impl worth_query_decl::facade::application_schema::fragment::Fragment for ForeignMemberSchema {
    const NAME: &'static str = "ForeignMemberSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (TopologyContribution, ForeignMemberContribution);
}
impl ApplicationSchema for ForeignMemberSchema {
    const OWNER: &'static str = "worth.query.certification.contribution-denials";
    const NAME: &'static str = "ForeignMemberSchema";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for ForeignMemberSchema {
    type Contributions = (ForeignMemberContribution, TopologyContribution);
}

fn inventory_drift_precedes_callbacks() {
    let setup_calls = Arc::new(AtomicUsize::new(0));
    let seed_calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory_program(
        validated_denial_program::<MissingContributionSchema>(),
        MissingContributionSchema::declaration().unwrap(),
        (topology_configuration(&setup_calls),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits(),
        |_, _, _| {
            seed_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::ContributionInventoryMismatch,
    );
    let result = application_installation::in_memory_program(
        validated_denial_program::<DuplicateContributionSchema>(),
        DuplicateContributionSchema::declaration().unwrap(),
        (
            topology_configuration(&setup_calls),
            Arc::clone(&setup_calls),
            Arc::clone(&setup_calls),
        ),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits(),
        |_, _, _| {
            seed_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::ContributionInventoryMismatch,
    );
    assert_eq!(
        setup_calls.load(Ordering::SeqCst),
        0,
        "inventory denial must precede all contribution callbacks"
    );
    assert_eq!(
        seed_calls.load(Ordering::SeqCst),
        0,
        "inventory denial must precede initial state"
    );
}

fn missing_handler_precedes_initializer() {
    let setup_calls = Arc::new(AtomicUsize::new(0));
    let seed_calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory_program(
        validated_denial_program::<MissingHandlerSchema>(),
        MissingHandlerSchema::declaration().unwrap(),
        (Arc::clone(&setup_calls), Arc::clone(&setup_calls)),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits(),
        |_bootstrap_phase, graph, installed| {
            seed_calls.fetch_add(1, Ordering::SeqCst);
            let binding = installed
                .installed_mutation_binding::<PlanarMutationBinding<MissingHandlerSchema>>()
                .expect("the initializer could resolve the omitted handler binding");
            graph.install_handler(&binding, PlanarHandler)
        },
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::MissingMutationHandler,
    );
    assert_eq!(
        setup_calls.load(Ordering::SeqCst),
        2,
        "both owned configurations must run before completeness is checked"
    );
    assert_eq!(
        seed_calls.load(Ordering::SeqCst),
        0,
        "initial state must not repair a contribution's missing handler"
    );
}

fn foreign_member_registration_is_denied() {
    for attempted_member in [ForeignMember::Handler, ForeignMember::Invariant] {
        let setup_calls = Arc::new(AtomicUsize::new(0));
        let seed_calls = Arc::new(AtomicUsize::new(0));
        let result = application_installation::in_memory_program(
            validated_denial_program::<ForeignMemberSchema>(),
            ForeignMemberSchema::declaration().unwrap(),
            (
                (attempted_member, Arc::clone(&setup_calls)),
                topology_configuration(&setup_calls),
            ),
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            limits(),
            |_, _, _| {
                seed_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        );
        assert_contribution_denial(
            result,
            WorthQueryPrimaryGraphInstallationDenialKind::ContributionMemberMismatch,
        );
        assert_eq!(
            setup_calls.load(Ordering::SeqCst),
            1,
            "foreign member attempt must fail before topology configuration"
        );
        assert_eq!(seed_calls.load(Ordering::SeqCst), 0);
    }
}

struct HandlerOmissionContribution;
impl<Schema: TopologySchemaBinding> ApplicationSchemaContribution<Schema>
    for HandlerOmissionContribution
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<Schema>>::IDENTITY;
    type Members = <TopologyContribution as ApplicationSchemaContribution<Schema>>::Members;
}
impl<Schema: TopologySchemaBinding> WorthQueryApplicationContribution<Schema>
    for HandlerOmissionContribution
{
    type Configuration = Arc<AtomicUsize>;
    fn configure(
        calls: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        setup.invariant(
            PositivePlanarTurn::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            |_| -> Result<PositiveTurnRule<Schema>, String> {
                panic!("missing handler must be denied before invariant factory execution")
            },
        )
    }
}

#[derive(Clone, Copy)]
enum ForeignMember {
    Handler,
    Invariant,
}
struct ForeignMemberContribution;
impl<Schema: ParameterSchemaBinding> ApplicationSchemaContribution<Schema>
    for ForeignMemberContribution
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <ParameterContribution as ApplicationSchemaContribution<Schema>>::IDENTITY;
    type Members = <ParameterContribution as ApplicationSchemaContribution<Schema>>::Members;
}
impl<Schema: ParameterSchemaBinding + TopologySchemaBinding>
    WorthQueryApplicationContribution<Schema> for ForeignMemberContribution
{
    type Configuration = (ForeignMember, Arc<AtomicUsize>);
    fn configure(
        (member, calls): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        calls.fetch_add(1, Ordering::SeqCst);
        match member {
            ForeignMember::Handler => {
                setup.handler::<PlanarMutationBinding<Schema>, _>(PlanarHandler)
            }
            ForeignMember::Invariant => setup.invariant(
                PositivePlanarTurn::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
                |_| -> Result<PositiveTurnRule<Schema>, String> {
                    panic!("foreign member factory must never execute")
                },
            ),
        }
    }
}

fn topology_configuration(calls: &Arc<AtomicUsize>) -> TopologyConfiguration {
    TopologyConfiguration {
        final_feature: worth_query_topology_entry::planar_final_feature_runtime(),
        setup_calls: Arc::clone(calls),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        invariant_read_observer: None,
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: Arc::new(AtomicBool::new(false)),
    }
}

fn limits() -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::new(
        super::resources::world_resources(),
        runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(32).expect("concurrent graph work is non-zero"),
        ),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

fn assert_contribution_denial<Schema>(
    result: Result<
        application_installation::WorthQueryProgramApplicationRuntime<
            Schema,
            DenialProgram<Schema>,
        >,
        WorthQueryApplicationPreparationDenial,
    >,
    expected: WorthQueryPrimaryGraphInstallationDenialKind,
) where
    Schema: ApplicationSchemaComposition,
{
    match result {
        Err(WorthQueryApplicationPreparationDenial::Contributions(denial)) => {
            assert_eq!(denial.kind(), expected, "{denial:?}")
        }
        Err(other) => panic!("expected contribution denial {expected:?}, got {other:?}"),
        Ok(_) => panic!("invalid contribution setup published an application"),
    }
}
