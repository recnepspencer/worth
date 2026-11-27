//! The request program: the birth body action, the three native control
//! actions and the `BornAtAdmission` invariant, opened over a held trusted
//! clock. The owner holds every acme grant and, at globex, every grant but
//! the subject read; the outsider holds the acme command grants only.
use std::time::Duration;

use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::{
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
};
use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::{
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
        WorthQueryFeatureDefinition,
    },
    application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::*,
    worth_query_application_contribution, worth_query_application_program,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::control::AuthorBinding;
use super::creator::{open_request_operation, OpenRequestBinding, RequestOpened};
use super::grants::{self, ADVANCE, AUTHOR, ENUMERATE, OPEN, READ, REVIEW, START};
use super::invariant::{self, BornAtAdmission};
use super::schema::*;
use super::subject_control::{AdvanceBinding, ReviewBinding, StartBinding};
use super::{control, disclosure, subject_control, COMPANY};

worth_query_decl::facade::worth_query_authentication_method!(BirthMethod);
worth_query_decl::facade::worth_query_authentication_audience!(BirthAudience);
worth_query_decl::facade::worth_query_principal_issuer!(BirthIssuer);

pub struct BirthFeature;
impl ApplicationFeature<BirthSchema> for BirthFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "workflow-birth-time.dx.feature.v1";
}

worth_query_application_contribution! {
    pub feature BirthActions<BirthSchema> for BirthFeature => BirthRuntime {
        query(ReadRequestBinding) { definition: ReadRequestQuery::definition(); };
        body_action(OpenRequestBinding, RequestOpened) as OpenRequestAction {
            operation: open_request_operation();
        };
        invariant(BornAtAdmission) {
            definition: invariant::definition();
            rule: invariant::resolve;
        };
    }
}

/// The body feature plus the control operations, their native actions and
/// the four capability contracts the workflow vocabulary binds.
pub struct BirthDefinition;

type Actions = BirthActions;

impl ApplicationFeatureDefinition<BirthSchema> for BirthDefinition {
    type Outputs = <Actions as ApplicationFeatureDefinition<BirthSchema>>::Outputs;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<BirthSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<BirthSchema> {
        let schema = <Actions as ApplicationFeatureDefinition<BirthSchema>>::members(schema);
        let schema = disclosure::declare(grants::declare(control::declare(schema)));
        subject_control::declare(schema)
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<BirthSchema, BirthFeature>()
            .action::<OpenRequestBinding, RequestOpened>()
            .finish_action()
            .native_action::<AuthorBinding>()
            .finish_action()
            .native_action::<StartBinding>()
            .finish_action()
            .native_action::<ReviewBinding>()
            .finish_action()
            .native_action::<AdvanceBinding>()
            .finish_action()
            .finish()]
    }
}

impl Registration<BirthSchema> for BirthDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<BirthSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<BirthSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(<Self as ApplicationFeatureDefinition<BirthSchema>>::declare_members(builder))
    }
}

impl WorthQueryFeatureDefinition<BirthSchema> for BirthDefinition {
    type Configuration = BirthRuntime;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<BirthSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<BirthSchema>>::contracts(contracts)
    }
    fn configure(
        runtime: BirthRuntime,
        setup: &mut WorthQueryApplicationContributionSetup<'_, BirthSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<BirthSchema>>::configure(runtime, setup)
    }
}

worth_query_application_program! {
    pub program BirthProgram for BirthSchema {
        identity: "workflow-birth-time.dx.program.v1",
        rules: ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationLocalRuleRef<BirthSchema, BirthFeature, BornAtAdmission>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleLeaf,
        >,
        contribution: BirthContribution {
            identity: "workflow-birth-time.dx.contribution.v1",
            members: [Records, OwnerBinding],
        },
        feature_definition: BirthDefinition,
    }
}

pub type BirthHome = WorthQueryApplicationHome<BirthSchema, BirthProgram>;

const OWNER: &str = "owner";
const OUTSIDER: &str = "outsider";
/// A second company where the owner may open requests but not read them.
pub const OTHER_COMPANY: &str = "globex";

fn identity(principal: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(BirthIssuer, principal).unwrap()
}

fn key<Entity>(value: &str) -> WorthQueryApplicationEntityKey<BirthSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value.to_owned()).unwrap()
}

/// One active, unexpiring, undelegable grant of `action` over `company`.
fn grant(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BirthSchema>,
    grantee: &str,
    company: &str,
    action: &str,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let grant = format!("grant-{grantee}-{company}-{action}");
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(Grant::reference(), key(&grant))
            .field(GrantFacts::grant_id(), grant.clone())
            .field(GrantFacts::action(), action.to_owned())
            .field(GrantFacts::purpose(), "purchase-request".to_owned())
            .field(GrantFacts::status(), "active".to_owned())
            .field(GrantFacts::workflow(), company.to_owned())
            .field(GrantFacts::not_before(), 0_u64)
            .field(GrantFacts::not_after(), u64::MAX)
            .field(GrantFacts::delegation_limit(), 0_u64),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Grantee::reference(),
        &format!("{grant}-grantee"),
        key(grantee),
        key(&grant),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Grantor::reference(),
        &format!("{grant}-grantor"),
        key(OWNER),
        key(&grant),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        GrantResource::reference(),
        &format!("{grant}-resource"),
        key(&grant),
        key(company),
    ))
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<BirthSchema>,
    installed: &WorthQueryInstalledApplicationSchema<BirthSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(OwnerBinding::reference())
        .unwrap();
    let enabled = WorthQueryPrincipalMappingStatus::Enabled;
    for (number, principal) in [(1_u64, OWNER), (2, OUTSIDER)] {
        let key = WorthQueryApplicationPrincipalKey::new(principal).unwrap();
        graph.bind_principal(&binding, key, number, identity(principal), enabled)?;
    }
    for company in [COMPANY, OTHER_COMPANY] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Company::reference(), key(company))
                .field(CompanyFacts::company_id(), company.to_owned()),
        )?;
    }
    let commands = [AUTHOR, START, REVIEW, ADVANCE, OPEN];
    for action in commands.into_iter().chain([READ, ENUMERATE]) {
        grant(graph, OWNER, COMPANY, action)?;
    }
    for action in commands {
        grant(graph, OWNER, OTHER_COMPANY, action)?;
        grant(graph, OUTSIDER, COMPANY, action)?;
    }
    Ok(())
}

/// Opens an in-memory Home, or creates or reopens the Directory Home at
/// `directory`, whose trusted time is the held clock.
pub fn open(
    trusted_time: WorthQueryFixedRuntimeTimeSource,
    directory: Option<&std::path::Path>,
) -> BirthHome {
    let build = ApplicationBuildIdentity::parse("b1e70000-0000-4000-8000-b1e700000001").unwrap();
    let config = WorthQueryHomeConfig::new(build, (BirthRuntime::new(),))
        .with_initial_state(|_, graph, installed| seed(graph, installed))
        .with_authorization_time_source(trusted_time);
    match directory {
        None => WorthQueryApplicationHome::open_memory(config),
        Some(path) => WorthQueryApplicationHome::open_directory(path, config),
    }
    .unwrap_or_else(|denial| panic!("birth Home opens: {denial}"))
}

/// The owner authenticated as a local process, with the request scope it acts in.
pub struct Caller {
    pub principal: WorthQueryAuthenticatedExternalPrincipal<BirthSchema>,
    pub scope: WorthQueryRequestScope,
}

pub fn owner(schema: &WorthQueryInstalledApplicationSchema<BirthSchema>) -> Caller {
    caller(schema, OWNER)
}

/// A principal holding the acme command grants but no subject read.
pub fn outsider(schema: &WorthQueryInstalledApplicationSchema<BirthSchema>) -> Caller {
    caller(schema, OUTSIDER)
}

fn caller(schema: &WorthQueryInstalledApplicationSchema<BirthSchema>, name: &str) -> Caller {
    let clock = WorthQueryAdmissionClock::system();
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    let principal = WorthQueryLocalProcessPrincipal::new(
        identity(name),
        BirthAudience,
        BirthMethod,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(schema, clock)
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates");
    Caller { principal, scope }
}
