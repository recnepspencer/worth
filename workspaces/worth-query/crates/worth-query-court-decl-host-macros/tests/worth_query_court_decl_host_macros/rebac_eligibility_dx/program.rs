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
            action publish(PublishDocumentBinding, DocumentPublished, PublishDocumentHandler)
                as PublishAction {};
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
/// A second site with its own administrator and no shared accounts.
pub const OTHER_SITE: &str = "S2";
/// Published documents; `D1` carries the seeded document roles.
pub const DOCUMENTS: [&str; 5] = ["D1", "D2", "D3", "D4", "D5"];

/// Seeded operators: (subject, operator id, enabled, site).
const OPERATORS: [(&str, u64, bool, &str); 7] = [
    ("admin", 1, true, SITE),
    ("alice", 2, true, SITE),
    ("bob", 3, true, SITE),
    ("carol", 4, true, SITE),
    ("dave", 5, false, SITE),
    ("erin", 6, true, SITE),
    ("admin2", 7, true, OTHER_SITE),
];

/// Seeded document roles: (key, holder, role, document).
const DOCUMENT_ROLES: [(&str, &str, &str, &str); 4] = [
    ("alice-approves-d1", "alice", "approver", "D1"),
    ("bob-authors-d1", "bob", "author", "D1"),
    ("bob-approves-d1", "bob", "approver", "D1"),
    ("dave-approves-d1", "dave", "approver", "D1"),
];

/// Seeded site roles: (key, holder, role, site).
const SITE_ROLES: [(&str, &str, &str, &str); 3] = [
    ("admin-site", "admin", "admin", SITE),
    ("carol-reads-site", "carol", "reader", SITE),
    ("admin2-site", "admin2", "admin", OTHER_SITE),
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
    for site in [SITE, OTHER_SITE] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Site::reference(), key(site))
                .field(SiteFacts::site_id(), site.to_owned()),
        )?;
    }
    for (subject, id, enabled, site) in OPERATORS {
        let status = if enabled {
            WorthQueryPrincipalMappingStatus::Enabled
        } else {
            WorthQueryPrincipalMappingStatus::Disabled
        };
        accounts::seed_principal(
            graph,
            installed,
            site,
            subject,
            id,
            identity(subject),
            status,
        )?;
    }
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
    for (assignment, holder, role, document) in DOCUMENT_ROLES {
        seed_assignment(graph, assignment, holder, role)?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Covers::reference(),
            format!("covers-{assignment}"),
            key(assignment),
            key(document),
        ))?;
    }
    for (assignment, holder, role, site) in SITE_ROLES {
        seed_assignment(graph, assignment, holder, role)?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Administers::reference(),
            format!("administers-{assignment}"),
            key(assignment),
            key(site),
        ))?;
    }
    Ok(())
}

/// One active assignment held by `holder`.
fn seed_assignment(
    graph: &mut WorthQueryPrimaryGraphBootstrap<QmsSchema>,
    assignment: &str,
    holder: &str,
    role: &str,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
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
    ))
}

fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse("9a3e9a3e-9a3e-4a3e-8a3e-9a3e9a3e9a3e").unwrap()
}

fn runtime() -> QmsRuntime {
    QmsRuntime::new(
        AssignRoleHandler,
        RevokeAssignmentHandler,
        PublishDocumentHandler,
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
