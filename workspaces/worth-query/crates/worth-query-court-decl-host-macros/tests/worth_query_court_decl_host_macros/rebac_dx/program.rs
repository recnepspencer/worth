//! The case board program: two ability-gated body actions and a gated read,
//! with every ability installed from the role matrix.
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

use super::policy::case_rule;
use super::schema::*;

worth_query_decl::facade::worth_query_authentication_method!(RebacMethod);
worth_query_decl::facade::worth_query_authentication_audience!(RebacAudience);
worth_query_decl::facade::worth_query_principal_issuer!(RebacIssuer);

pub struct RebacFeature;
impl ApplicationFeature<RebacSchema> for RebacFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "rebac.dx.feature.v1";
}

worth_query_application_contribution! {
    pub feature CaseActions<RebacSchema> for RebacFeature => RebacRuntime {
        query(ReadCaseBinding) { definition: ReadCaseQuery::definition(); };
        body_action(AnnotateCaseBinding, CaseAnnotated) as AnnotateAction {
            operation: annotate_operation();
        };
        body_action(CloseCaseBinding, CaseClosed) as CloseAction {
            operation: close_operation();
        };
    }
}

/// The actions plus the authorization the role matrix installs.
pub struct RebacDefinition;
type Actions = CaseActions;

impl ApplicationFeatureDefinition<RebacSchema> for RebacDefinition {
    type Outputs = <Actions as ApplicationFeatureDefinition<RebacSchema>>::Outputs;
    fn members(
        schema: ApplicationSchemaDeclarationBuilder<RebacSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<RebacSchema> {
        <Actions as ApplicationFeatureDefinition<RebacSchema>>::members(schema)
            .policy(CaseAccess::reference())
            .role_abilities(CaseAccess::reference(), case_rule)
            .operation_requires_ability(
                AnnotateCaseOp::reference(),
                case_ability::Annotate::reference(),
            )
            .operation_requires_ability(CloseCaseOp::reference(), case_ability::Close::reference())
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <Actions as ApplicationFeatureDefinition<RebacSchema>>::feature_specs()
    }
}

impl Registration<RebacSchema> for RebacDefinition {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RebacSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<RebacSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(<Self as ApplicationFeatureDefinition<RebacSchema>>::declare_members(builder))
    }
}

impl WorthQueryFeatureDefinition<RebacSchema> for RebacDefinition {
    type Configuration = RebacRuntime;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<RebacSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<RebacSchema>>::contracts(contracts)
    }
    fn configure(
        runtime: RebacRuntime,
        setup: &mut WorthQueryApplicationContributionSetup<'_, RebacSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        <Actions as WorthQueryFeatureDefinition<RebacSchema>>::configure(runtime, setup)
    }
}

worth_query_application_program! {
    pub program RebacProgram for RebacSchema {
        identity: "rebac.dx.program.v1",
        contribution: RebacContribution {
            identity: "rebac.dx.contribution.v1",
            members: [Records, UserBinding],
        },
        feature_definition: RebacDefinition,
    }
}

pub type RebacHome = WorthQueryApplicationHome<RebacSchema, RebacProgram>;

/// `(user, id, role, level, site)`: each user holds one assignment.
pub const USERS: [(&str, u64, StaffRole, u64, u64); 6] = [
    ("manager", 1, StaffRole::Manager, 2, 1),
    ("junior", 2, StaffRole::Manager, 1, 1),
    ("reviewer", 3, StaffRole::Reviewer, 3, 1),
    ("auditor", 4, StaffRole::Auditor, 3, 1),
    ("elsewhere", 5, StaffRole::Manager, 3, 2),
    ("author", 6, StaffRole::Manager, 3, 1),
];
/// `(case, site, author)`: case 10 by `reviewer`, case 11 by `author`.
pub const CASES: [(u64, u64, &str); 2] = [(10, 1, "reviewer"), (11, 1, "author")];

fn key<E>(text: &str) -> WorthQueryApplicationEntityKey<RebacSchema, E> {
    WorthQueryApplicationEntityKey::new(text).expect("fixture key")
}

fn identity(name: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(RebacIssuer, name).expect("fixture identity")
}

fn user_id(name: &str) -> u64 {
    USERS
        .iter()
        .find(|(user, ..)| *user == name)
        .map_or(0, |(_, id, ..)| *id)
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<RebacSchema>,
    installed: &WorthQueryInstalledApplicationSchema<RebacSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(UserBinding::reference())
        .expect("principal binding installs");
    for site in [1_u64, 2] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Site::reference(), key(&format!("site-{site}")))
                .field(SiteFacts::site_id(), site),
        )?;
    }
    for (user, id, role, level, site) in USERS {
        let principal = WorthQueryApplicationPrincipalKey::new(user).expect("fixture principal");
        let status = WorthQueryPrincipalMappingStatus::Enabled;
        graph.bind_principal(&binding, principal, id, identity(user), status)?;
        let assignment = format!("assignment-{user}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Assignment::reference(), key(&assignment))
                .field(AssignmentFacts::role(), role)
                .field(AssignmentFacts::level(), level),
        )?;
        let held = format!("holds-{user}");
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            UserAssignments::reference(),
            held,
            key(user),
            key(&assignment),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            AssignmentSite::reference(),
            format!("at-{user}"),
            key(&assignment),
            key(&format!("site-{site}")),
        ))?;
    }
    for (case, site, author) in CASES {
        let name = format!("case-{case}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Case::reference(), key(&name))
                .field(CaseFacts::case_id(), case)
                .field(CaseFacts::author_id(), user_id(author))
                .field(CaseFacts::note(), 0),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            CaseSite::reference(),
            format!("{name}-site"),
            key(&name),
            key(&format!("site-{site}")),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            CaseAuthor::reference(),
            format!("{name}-author"),
            key(&name),
            key(author),
        ))?;
    }
    Ok(())
}

pub fn open(path: &std::path::Path, clock: &WorthQueryAdmissionClock) -> RebacHome {
    let build = ApplicationBuildIdentity::parse("9a1d9a1d-9a1d-4a1d-8a1d-9a1d9a1d9a1d").unwrap();
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build, (RebacRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("case board Home opens: {denial}"))
}

/// Authenticates the fixture user `name` as a local process.
pub fn principal(
    home: &RebacHome,
    clock: &WorthQueryAdmissionClock,
    name: &str,
) -> WorthQueryAuthenticatedExternalPrincipal<RebacSchema> {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity(name),
        RebacAudience,
        RebacMethod,
        Duration::from_secs(600),
    )
    .expect("local principal")
    .admit(home.primary_graph().installed_schema(), clock.clone())
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates")
}
