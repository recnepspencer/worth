use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_query_decl::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity, ApplicationSchema,
    ApplicationSchemaComposition, ApplicationSchemaContribution,
    ApplicationSchemaContributionAuthoring, ApplicationSchemaContributionIdentity,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    application_installation::{
        self, WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
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
}

// The same real entry declarations are retained while each fixture varies only
// the contribution boundary under attack.
macro_rules! boundary_schema {
    ($Schema:ident, [$Topology:ty, $Parameter:ty], [$($Configured:ty),+]) => {
        struct $Schema;
        impl TopologySchemaBinding for $Schema {}
        impl ParameterSchemaBinding for $Schema {}
        impl ApplicationSchema for $Schema {
            const OWNER: &'static str = "worth.query.certification.contribution-denials";
            const NAME: &'static str = stringify!($Schema);
            const MAJOR: u32 = 1;
            const MINOR: u32 = 0;
            fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
                ApplicationSchemaContributionAuthoring::contributions(
                    ApplicationSchemaDeclarationBuilder::<Self>::for_schema(),
                ).register::<$Topology>()?.register::<$Parameter>()?.build()
            }
        }
        impl ApplicationSchemaComposition for $Schema {
            type Contributions = ($($Configured,)+);
        }
    };
}

boundary_schema!(
    MissingContributionSchema,
    [TopologyContribution, ParameterContribution],
    [TopologyContribution]
);
boundary_schema!(
    DuplicateContributionSchema,
    [TopologyContribution, ParameterContribution],
    [
        TopologyContribution,
        ParameterContribution,
        ParameterContribution
    ]
);
boundary_schema!(
    MissingHandlerSchema,
    [HandlerOmissionContribution, ParameterContribution],
    [HandlerOmissionContribution, ParameterContribution]
);
boundary_schema!(
    ForeignMemberSchema,
    [TopologyContribution, ForeignMemberContribution],
    [ForeignMemberContribution, TopologyContribution]
);

fn inventory_drift_precedes_callbacks() {
    let setup_calls = Arc::new(AtomicUsize::new(0));
    let seed_calls = Arc::new(AtomicUsize::new(0));
    let result = application_installation::in_memory::<MissingContributionSchema>(
        MissingContributionSchema::declaration().unwrap(),
        (topology_configuration(&setup_calls),),
        limits(),
        |_, _| {
            seed_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );
    assert_contribution_denial(
        result,
        WorthQueryPrimaryGraphInstallationDenialKind::ContributionInventoryMismatch,
    );
    let result = application_installation::in_memory::<DuplicateContributionSchema>(
        DuplicateContributionSchema::declaration().unwrap(),
        (
            topology_configuration(&setup_calls),
            Arc::clone(&setup_calls),
            Arc::clone(&setup_calls),
        ),
        limits(),
        |_, _| {
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
    let result = application_installation::in_memory::<MissingHandlerSchema>(
        MissingHandlerSchema::declaration().unwrap(),
        (Arc::clone(&setup_calls), Arc::clone(&setup_calls)),
        limits(),
        |graph, installed| {
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
        let result = application_installation::in_memory::<ForeignMemberSchema>(
            ForeignMemberSchema::declaration().unwrap(),
            (
                (attempted_member, Arc::clone(&setup_calls)),
                topology_configuration(&setup_calls),
            ),
            limits(),
            |_, _| {
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
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        <TopologyContribution as ApplicationSchemaContribution<Schema>>::register_members(builder)
    }
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
            |_| -> Result<PositiveTurnRule, String> {
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
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        <ParameterContribution as ApplicationSchemaContribution<Schema>>::register_members(builder)
    }
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
                |_| -> Result<PositiveTurnRule, String> {
                    panic!("foreign member factory must never execute")
                },
            ),
        }
    }
}

fn topology_configuration(calls: &Arc<AtomicUsize>) -> TopologyConfiguration {
    TopologyConfiguration {
        setup_calls: Arc::clone(calls),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
    }
}

fn limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        super::resources::world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(4096, 8192, 4096).unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(4096, 4096, 4096, 32).unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

fn assert_contribution_denial<Schema>(
    result: Result<
        primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryInMemoryApplicationDenial,
    >,
    expected: WorthQueryPrimaryGraphInstallationDenialKind,
) {
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => {
            assert_eq!(denial.kind(), expected)
        }
        Err(other) => panic!("expected contribution denial {expected:?}, got {other:?}"),
        Ok(_) => panic!("invalid contribution setup published an application"),
    }
}
