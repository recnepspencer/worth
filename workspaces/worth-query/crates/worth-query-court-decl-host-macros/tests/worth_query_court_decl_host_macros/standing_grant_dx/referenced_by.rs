//! W10B referenced-by, positively: "which grants reference this site?" is an
//! installed query stepping backward from the scoped site along `GrantSite`.
//! It returns the referencing rows the principal may see at the head, the
//! same rows at a pin taken before a new grant links, and the entity impact
//! advertises it as the relation's reverse read.
use worth_query_decl::facade::{
    application_query::{ApplicationQueryDefinitionBuilder, ApplicationQueryRootPath},
    WorthQuery, WorthQueryRow,
};
use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;

use super::observers::home;
use super::program::{caller, QmsHome};
use super::schema::*;
use super::{at, temp_home_path};

/// One grant referencing a site.
#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = QmsSchema, root = RoleGrant, namespace = QMS, member)]
pub struct SiteGrantRow {
    #[worth_query(field = GrantId)]
    pub grant_id: String,
}

/// Every grant standing over the scoped site.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(schema = QmsSchema, context = Desktop, namespace = QMS, member,
    result = SiteGrantRow, root = RoleGrant, limits = 16)]
pub struct SiteGrants {
    #[worth_query(scope = SiteId)]
    pub site: String,
}

pub fn site_grants(
) -> ApplicationQueryDefinitionBuilder<QmsSchema, SiteGrantsQuery, (), SiteGrantRow, Site> {
    SiteGrantsQuery::definition().root_path(
        ApplicationQueryRootPath::from(Site::reference()).reverse(GrantSite::reference()),
    )
}

fn sorted(mut ids: Vec<String>) -> Vec<String> {
    ids.sort();
    ids
}

fn grant_ids(rows: &[SiteGrantRow]) -> Vec<String> {
    sorted(rows.iter().map(|row| row.grant_id.clone()).collect())
}

/// The six seeded grants of `site`, all held by alice.
fn seeded(site: &str) -> Vec<String> {
    sorted(
        [
            "review",
            "approve",
            "audit",
            "witness",
            "observe",
            "countersign",
        ]
        .iter()
        .map(|role| format!("{site}-{role}"))
        .collect(),
    )
}

fn link_bob_review(home: &QmsHome) {
    home.session(caller(home, "alice").principal)
        .run(AssignRole {
            site: "north".to_owned(),
            member: 2,
            grant: "bob-review".to_owned(),
        })
        .unwrap_or_else(|denial| panic!("alice grants bob: {denial:?}"));
}

#[test]
fn referenced_by_returns_the_referencing_rows_at_the_head_and_at_a_pin() {
    let path = temp_home_path("referenced-by");
    let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    let session = home.session(caller(&home, "bob").principal);
    let north = || SiteGrants {
        site: "north".to_owned(),
    };

    let referenced = session.referenced_by(north()).expect("bob may read north");
    assert_eq!(referenced.referenced_entity(), "Site");
    let relations: Vec<_> = referenced
        .relations()
        .iter()
        .map(|edge| (edge.relation(), edge.from(), edge.to()))
        .collect();
    assert_eq!(relations, [("GrantSite", "RoleGrant", "Site")]);
    assert_eq!(grant_ids(referenced.rows()), seeded("north"));
    let south = session
        .referenced_by(SiteGrants {
            site: "south".to_owned(),
        })
        .expect("bob may read south");
    assert_eq!(grant_ids(south.rows()), seeded("south"));

    // A pin taken before a new grant links keeps reading the older rows.
    let before = session.snapshot().expect("the head pins");
    link_bob_review(&home);
    let mut after = seeded("north");
    after.push("bob-review".to_owned());
    let head = session.referenced_by(north()).expect("bob may read north");
    assert_eq!(grant_ids(head.rows()), sorted(after));
    let pinned = before.referenced_by(north()).expect("the pin is readable");
    assert_eq!(grant_ids(pinned.rows()), seeded("north"));

    // Impact advertises exactly this query as GrantSite's reverse read.
    let impact = session.entity_impact(Site::reference());
    let grant_site = impact
        .incoming()
        .iter()
        .find(|incoming| incoming.edge().relation() == "GrantSite")
        .expect("GrantSite is incoming to Site");
    assert_eq!(grant_site.reverse_reads(), ["site_grants"]);
}
