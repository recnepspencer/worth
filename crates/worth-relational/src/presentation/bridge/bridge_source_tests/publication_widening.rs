use std::sync::Arc;

use worth_proof::TransitionOutcome;

use crate::tests::support::create_entity_outcome;

use super::super::RuntimeBridgeRelationalSource;
use super::support::runtime_with_test_schema;

#[test]
fn live_runtime_mints_publication_provenance_and_rejects_foreign_widening_authority() {
    let owner = runtime_with_test_schema();
    create_entity_outcome(&owner, "owner");
    let commit = owner
        .publication()
        .latest_bundle()
        .unwrap()
        .commit
        .commit_id;
    let branch_identity = owner
        .branch_identity(&crate::history::data::BranchId("main".to_owned()))
        .expect("owner branch identity");
    let admission = owner.admit_opaque_aspect_bridge_widening("model").unwrap();
    let wrong_role_admission = owner
        .admit_opaque_aspect_bridge_widening("analysis")
        .unwrap();
    let foreign = runtime_with_test_schema();
    let foreign_admission = foreign
        .admit_opaque_aspect_bridge_widening("model")
        .unwrap();
    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(owner), "model")
        .expect("owner graph source");
    let (_, basis) = source
        .observe_branch_basis(&branch_identity)
        .expect("owner exact basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("owner retained observation");

    assert!(matches!(
        source
            .publish_commit_with_widening_at_snapshot(
                commit,
                lease.snapshot_identity(),
                &foreign_admission,
            )
            .expect("selected commit"),
        TransitionOutcome::Stale(super::super::RelationalBridgePublicationStale::RuntimeAuthority)
    ));
    assert!(matches!(
        source
            .publish_commit_with_widening_at_snapshot(
                commit,
                lease.snapshot_identity(),
                &wrong_role_admission,
            )
            .expect("selected commit"),
        TransitionOutcome::RebindRequired(
            super::super::RelationalBridgePublicationRebindRequired::GraphRole
        )
    ));
    let TransitionOutcome::Success(publication) = source
        .publish_commit_with_widening_at_snapshot(commit, lease.snapshot_identity(), &admission)
        .expect("selected commit")
    else {
        panic!("owner-minted publication authority should publish its exact commit");
    };

    assert_eq!(publication.commit_id(), commit);
    assert_eq!(publication.graph_role(), "model");
    assert!(publication.runtime_instance_id() > 0);
    assert_eq!(
        publication.adapter_semantic_identity(),
        "worth-relational-bridge-adapter-v1"
    );
    assert!(publication
        .source_basis()
        .contains(&format!("commit={}", commit.0)));
    let provenance = publication
        .bridge_envelope()
        .producer_metadata()
        .authoritative_source()
        .expect("owner publication carries its source authority into the Bridge envelope");
    assert_eq!(
        provenance.runtime_instance_id(),
        publication.runtime_instance_id()
    );
    assert_eq!(provenance.graph_role(), publication.graph_role());
    assert_eq!(
        provenance.adapter_semantic_identity(),
        publication.adapter_semantic_identity()
    );
    assert_eq!(provenance.source_basis(), publication.source_basis());
}
