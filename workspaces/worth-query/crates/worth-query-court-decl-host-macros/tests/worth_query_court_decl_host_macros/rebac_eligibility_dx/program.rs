//! The QMS program: the registry and access join the contribution as members,
//! and the seeded site holds one operator per verdict an approver can reach.
use std::path::Path;
use std::time::Duration;

use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_installation::ApplicationBuildIdentity,
    domain::WorthQueryInstalledApplicationSchema, primary_graph::*,
    worth_query_application_program, WorthQueryApplicationHome, WorthQueryHomeConfig,
};

use super::roles::*;
use super::schema::*;

worth_query_decl::facade::worth_query_principal_issuer!(pub QmsIssuer);
worth_query_decl::facade::worth_query_authentication_audience!(pub QmsAudience);
worth_query_decl::facade::worth_query_authentication_method!(pub LocalProcess);

worth_query_application_program! {
    pub program QmsProgram for QmsSchema {
        identity: "rebac.qms.dx.program.v1",
        contribution: QmsContribution {
            identity: "rebac.qms.dx.contribution.v1",
            members: [Records, accounts::RegistryMembers, QmsAccess],
        },
        feature: QmsFeature => QmsDefinition / QmsRuntime {
            identity: "rebac.qms.dx.feature.v1",
            query(SiteDocumentsBinding) { definition: SiteDocumentsQuery::definition(); };
            body_action(ApproveDocumentBinding, DocumentApproved) as ApproveAction {
                operation: approve_document_operation();
            };
            action assign(AssignDocumentRoleBinding, DocumentRoleAssigned, AssignRoleHandler)
                as AssignAction {};
            action revoke(RevokeAssignmentBinding, AssignmentRevoked, RevokeAssignmentHandler)
                as RevokeAction {};
            action onboard(
                accounts::OnboardPrincipalBinding,
                accounts::PrincipalOnboarded,
                accounts::OnboardPrincipalHandler
            ) as OnboardAction {};
            action status(
                accounts::SetPrincipalStatusBinding,
                accounts::PrincipalStatusSet,
                accounts::SetPrincipalStatusHandler
            ) as StatusAction {};
        }
    }
}

pub type QmsHome = WorthQueryApplicationHome<QmsSchema, QmsProgram>;
pub type Principal = WorthQueryAuthenticatedExternalPrincipal<QmsSchema>;

pub const SITE: &str = "S1";
/// Published documents; `D1` carries the seeded document roles.
pub const DOCUMENTS: [&str; 5] = ["D1", "D2", "D3", "D4", "D5"];

/// Seeded operators: (subject, operator id, enabled).
const OPERATORS: [(&str, u64, bool); 6] = [
    ("admin", 1, true),
    ("alice", 2, true),
    ("bob", 3, true),
    ("carol", 4, true),
    ("dave", 5, false),
    ("erin", 6, true),
];

/// Seeded assignments: (key, holder, role, covered document or the site).
const ASSIGNMENTS: [(&str, &str, &str, Option<&str>); 6] = [
    ("admin-site", "admin", "admin", None),
    ("alice-approves-d1", "alice", "approver", Some("D1")),
    ("bob-authors-d1", "bob", "author", Some("D1")),
    ("bob-approves-d1", "bob", "approver", Some("D1")),
    ("carol-reads-site", "carol", "reader", None),
    ("dave-approves-d1", "dave", "approver", Some("D1")),
];

pub fn identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(QmsIssuer, subject).unwrap()
}

fn key<Entity>(value: &str) -> WorthQueryApplicationEntityKey<QmsSchema, Entity> {
    WorthQueryApplicationEntityKey::new(value.to_owned()).unwrap()
}

fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<QmsSchema>,
    installed: &WorthQueryInstalledApplicationSchema<QmsSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let registry = installed
        .principal_binding(accounts::Registry::reference())
        .unwrap();
    for (subject, id, enabled) in OPERATORS {
        let status = if enabled {
            WorthQueryPrincipalMappingStatus::Enabled
        } else {
            WorthQueryPrincipalMappingStatus::Disabled
        };
        let principal_key = WorthQueryApplicationPrincipalKey::new(subject).unwrap();
        graph.bind_principal(&registry, principal_key, id, identity(subject), status)?;
    }
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(Site::reference(), key(SITE))
            .field(SiteFacts::site_id(), SITE.to_owned()),
    )?;
    for document in DOCUMENTS {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Document::reference(), key(document))
                .field(DocumentFacts::document_id(), document.to_owned())
                .field(DocumentFacts::approved(), false),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Publishes::reference(),
            format!("publishes-{document}"),
            key(SITE),
            key(document),
        ))?;
    }
    for (assignment, holder, role, document) in ASSIGNMENTS {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Assignment::reference(), key(assignment))
                .field(AssignmentFacts::assignment_id(), assignment.to_owned())
                .field(AssignmentFacts::role(), role.to_owned())
                .field(AssignmentFacts::active(), true),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Holds::reference(),
            format!("holds-{assignment}"),
            key(holder),
            key(assignment),
        ))?;
        match document {
            Some(document) => graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Covers::reference(),
                format!("covers-{assignment}"),
                key(assignment),
                key(document),
            ))?,
            None => graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Administers::reference(),
                format!("administers-{assignment}"),
                key(assignment),
                key(SITE),
            ))?,
        }
    }
    Ok(())
}

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse("9a3e9a3e-9a3e-4a3e-8a3e-9a3e9a3e9a3e").unwrap()
}

fn runtime() -> QmsRuntime {
    QmsRuntime::new(
        AssignRoleHandler,
        RevokeAssignmentHandler,
        accounts::OnboardPrincipalHandler,
        accounts::SetPrincipalStatusHandler,
    )
}

/// Opens (or reopens) the Directory Home at `path`, seeding a fresh one.
pub fn open(path: &Path, clock: &WorthQueryAdmissionClock) -> QmsHome {
    WorthQueryApplicationHome::open_directory(
        path,
        WorthQueryHomeConfig::new(build(), (runtime(),))
            .with_clock(clock.clone())
            .with_initial_state(|_, graph, installed| seed(graph, installed)),
    )
    .unwrap_or_else(|denial| panic!("QMS Home opens: {denial}"))
}

/// Authenticates `subject` as a local process; the graph decides whether it
/// maps to an enabled operator when it asks or acts.
pub fn principal(home: &QmsHome, clock: &WorthQueryAdmissionClock, subject: &str) -> Principal {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    WorthQueryLocalProcessPrincipal::new(
        identity(subject),
        QmsAudience,
        LocalProcess,
        Duration::from_secs(600),
    )
    .unwrap()
    .admit(home.primary_graph().installed_schema(), clock.clone())
    .expect("local principal admits")
    .authenticate_local(&scope)
    .expect("local principal authenticates")
}
