//! The workflow Home carries the plain Home's preview, correction-history and
//! impact surfaces, and a referenced-by read refuses a non-reverse query.
use super::*;
use document::schema::Document;
use worth_query_host::facade::{
    audit::{WorthQueryImpactDependency, WorthQueryImpactRole, WorthQueryReferencedByDenial},
    WorthQueryCorrectionHistoryLimit, WorthQueryPreviewDiscardDenial,
};

#[test]
fn workflow_home_forwards_what_if_previews_and_correction_history() {
    let trusted = trusted_now();
    let location = Location::Directory(super::super::super::super::fixture::location());
    let (home, authentication) = open(&location, &trusted);
    let (operator, _) = principals(&home);
    let original_branch = home.active_branch();
    set_retention(&home, &home.session(&operator), 2, 4501);

    let discarded = home.fork_preview("discarded what-if").unwrap();
    let scratch = home.session(&operator).on_branch(discarded.branch());
    set_retention(&home, &scratch, 8, 4502);
    drop(scratch);
    home.discard_preview(discarded).unwrap();
    assert_eq!(home.active_branch(), original_branch);
    assert_eq!(
        retention_at(&home.session(&operator).snapshot().unwrap()),
        2
    );

    let adopted = home.fork_preview("adopted what-if").unwrap();
    let branch = adopted.branch();
    let what_if = home.session(&operator).on_branch(branch);
    set_retention(&home, &what_if, 7, 4503);
    assert_eq!(
        retention_at(&home.session(&operator).snapshot().unwrap()),
        2
    );
    drop(what_if);
    assert_eq!(home.adopt_preview(adopted).unwrap(), original_branch);
    assert_eq!(home.active_branch(), branch);
    assert_eq!(
        retention_at(&home.session(&operator).snapshot().unwrap()),
        7
    );
    assert!(matches!(
        home.close_branch(branch),
        Err(WorthQueryPreviewDiscardDenial::ActiveBranchProtected)
    ));

    let limit = WorthQueryCorrectionHistoryLimit::new(8).unwrap();
    let history = home.correction_history(&operator, limit).unwrap();
    assert!(history.next_redo().is_none(), "{history:?}");
    drop(authentication);
    home.close_clean().unwrap();
}

#[test]
fn entity_impact_and_referenced_by_read_installed_meaning_first() {
    let trusted = trusted_now();
    let (home, _authentication) = open(&Location::MemoryOnly, &trusted);
    let (operator, reviewer) = principals(&home);
    let session = home.session(&operator);
    let impact = session.entity_impact(Document::reference());
    assert_eq!(impact.entity(), "Document");
    assert!(impact
        .incoming()
        .iter()
        .all(|incoming| incoming.edge().to() == "Document"));
    assert!(impact
        .outgoing()
        .iter()
        .all(|edge| edge.from() == "Document"));
    let scoped: Vec<&str> = impact
        .dependencies()
        .iter()
        .filter_map(|dependency| match dependency {
            WorthQueryImpactDependency::Capability {
                capability,
                role: WorthQueryImpactRole::ScopeEntity,
                ..
            } => Some(capability.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        scoped.contains(&"PayloadCapability") && scoped.contains(&"MetadataCapability"),
        "{impact:#?}"
    );

    // The subject read selects its scoped document itself, so it reads no
    // incoming link; it refuses before admission, for any principal.
    for principal in [&operator, &reviewer] {
        let refused = home.session(principal).referenced_by(SubjectInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
        });
        assert!(
            matches!(
                refused,
                Err(WorthQueryReferencedByDenial::NotAnIncomingReference { ref query })
                    if query == "history_subject_read"
            ),
            "{refused:?}"
        );
    }
}
