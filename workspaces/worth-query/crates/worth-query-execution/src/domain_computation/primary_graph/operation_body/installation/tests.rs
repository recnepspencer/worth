//! Owner-level evidence for the exact capacity gate called by body installation.
use super::{require_candidate_capacity, WorthQueryPrimaryGraphInstallationDenialKind as Kind};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling as Cardinality,
        ApplicationCandidateRequirements as Requirements,
        ApplicationCandidateResourceCeiling as Resources,
    },
    application_schema::{
        ApplicationOperationBodyCreation as Creation, ApplicationOperationBodyCreationLink as Link,
        ApplicationOperationBodyCreationLinkDirection as Direction,
        ApplicationOperationBodyCreationLinkPosture as Posture,
        ApplicationOperationBodyProgram as Body,
    },
};
#[test]
fn replacement_installation_capacity_requires_room_for_the_observed_old_edge() {
    let links = [
        ("History", Posture::Append),
        ("Current", Posture::ReplaceSingle),
    ]
    .into_iter()
    .map(|(name, posture)| Link {
        relation: name.into(),
        from: "Root".into(),
        to: "Item".into(),
        key: name.into(),
        direction: Direction::FromScope,
        posture,
    })
    .collect();
    let body = Body::from_untrusted_in_scope(
        "Root".into(),
        vec![],
        Some(
            Creation::from_untrusted("Item".into(), "new".into(), vec![])
                .with_untrusted_links(links),
        ),
        vec![],
    );
    let capacity = |unlinks| {
        Requirements::fixed_shape(
            Cardinality::fixed(1, 0, 2, unlinks, 0, 0),
            Resources::representation_bytes(4096),
        )
    };
    let denied = require_candidate_capacity(&body, &capacity(0), "test.replace").unwrap_err();
    assert_eq!(denied.kind(), Kind::OperationBodyUnavailable);
    assert_eq!(denied.subject(), "test.replace");
    assert!(require_candidate_capacity(&body, &capacity(1), "test.replace").is_ok());
}
