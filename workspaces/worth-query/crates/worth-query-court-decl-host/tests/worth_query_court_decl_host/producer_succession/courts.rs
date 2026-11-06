//! Literal branch rosters and generation observations through real producer settlement.
use super::{settlement::*, *};
#[test]
fn generator_shrink_retires_the_deleted_childs_owned_publications() {
    let app = world::install_two_levels();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let sibling = child_id(&first, "root.0");
    close(first, &request);
    resize(&app, &request, 1, 1);
    let shrunk = settle(&app, &request);
    assert_eq!(child_id(&shrunk, "root.0"), sibling);
    close(shrunk, &request);
    assert_absent(&request, &["root.1"]);
    assert_live(&request, &["root", "root.0"]);
}
#[test]
fn three_level_shrink_retires_exactly_the_removed_branch_and_regrows_fresh() {
    let selections = Selections::default();
    let app = world::install(2, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let old = child_id(&first, "root.1");
    let sibling = child_id(&first, "root.0");
    close(first, &request);
    resize(&app, &request, 1, 2);
    let shrunk = settle(&app, &request);
    assert_eq!(child_id(&shrunk, "root.0"), sibling);
    close(shrunk, &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    assert_live(&request, &["root", "root.0", "root.0.0"]);
    resize(&app, &request, 2, 3);
    let regrown = settle(&app, &request);
    assert_ne!(child_id(&regrown, "root.1"), old);
    assert_eq!(child_id(&regrown, "root.0"), sibling);
    close(regrown, &request);
    assert_live(
        &request,
        &["root", "root.0", "root.0.0", "root.1", "root.1.0"],
    );
    let selected = selections.lock().unwrap();
    let regrown = selected
        .iter()
        .filter(|s| s.key == "root.1")
        .collect::<Vec<_>>();
    assert_eq!(regrown.len(), 2);
    assert!(regrown
        .iter()
        .all(|s| s.tier == 1 && !s.preserve && !s.prior));
}

#[test]
fn settled_generator_roles_match_its_publication_and_closure_witnesses_are_separate() {
    use producers::parent::preserve::{Children, Output, Outputs, RetiredRealizations};
    use worth_relational::facade::{history::CommitId, identity::EntityId};
    type Role = (String, WorthQueryApplicationOutputPosture, EntityId);
    #[derive(Debug)]
    struct Observed {
        publication: CommitId,
        receipt: WorthQueryApplicationCommitReceipt,
        roles: Vec<Role>,
        children: Vec<(String, EntityId)>,
        realizations: Vec<(String, EntityId)>,
    }
    let app = world::install(2, Selections::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    close(first, &request);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&seen);
    let recorder = app
        .runtime()
        .record_output_publications_for_test(move |settlement| {
            let Ok(roles) = settlement.outputs_of::<Outputs>() else {
                return;
            };
            let receipt = settlement;
            let mut published = roles
                .family_entries::<Children>()
                .unwrap()
                .iter()
                .map(|entry| (entry.role().to_owned(), entry.posture(), entry.entity_id()))
                .collect::<Vec<_>>();
            published.push((
                "realization".to_owned(),
                WorthQueryApplicationOutputPosture::Preserve,
                roles.entity::<Output>().unwrap().entity_id(),
            ));
            assert!(
                roles
                    .family_entries::<RetiredRealizations>()
                    .unwrap()
                    .is_empty(),
                "foreign realizations are never canonical generator roles"
            );
            let closure = roles.closure_evidence();
            observed.lock().unwrap().push(Observed {
                publication: receipt.commit_reference().commit_id,
                receipt: receipt.clone(),
                roles: published,
                children: closure
                    .family_entries::<Children>()
                    .unwrap()
                    .into_iter()
                    .map(|entry| (entry.role.to_owned(), entry.entity_id))
                    .collect(),
                realizations: closure
                    .family_entries::<RetiredRealizations>()
                    .unwrap()
                    .into_iter()
                    .map(|entry| (entry.role.to_owned(), entry.entity_id))
                    .collect(),
            });
        });
    resize(&app, &request, 1, 31);
    let shrunk = settle(&app, &request);
    let final_roles = shrunk
        .settlement()
        .root_settlement()
        .outputs_of::<Outputs>()
        .unwrap();
    assert!(final_roles
        .family_entries::<RetiredRealizations>()
        .unwrap()
        .is_empty());
    assert!(shrunk
        .settlement()
        .root_settlement()
        .closure_evidence_of::<Outputs>()
        .is_ok());
    close(shrunk, &request);
    drop(recorder);
    let seen = seen.lock().unwrap();
    let policy = worth_foundational::facade::ExecutionRequestPolicy::new(
        worth_foundational::facade::ExecutionPosture::Serial,
        worth_foundational::facade::DeterminismContract::CanonicalBitwise,
        worth_foundational::facade::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
    );
    let payer = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&payer);
    for publication in seen
        .iter()
        .filter(|p| !p.children.is_empty() || !p.realizations.is_empty())
    {
        assert!(
            publication
                .receipt
                .output_correspondence()
                .checkpoint_roundtrip_equal_for_test(&execution)
                .unwrap(),
            "a shrink's live and checkpoint-restored canonical correspondence have equal identity"
        );
    }

    let retirement = seen
        .iter()
        .find(|entry| !entry.realizations.is_empty())
        .expect(
            "the real shrink publication carries closure evidence before the program refreshes its remaining children",
        );
    assert_eq!(
        retirement
            .roles
            .iter()
            .filter(|(_, p, _)| *p == WorthQueryApplicationOutputPosture::Preserve)
            .count(),
        2
    );
    assert_eq!(
        retirement
            .roles
            .iter()
            .filter(|(_, p, _)| *p == WorthQueryApplicationOutputPosture::Retire)
            .count(),
        1
    );
    assert_eq!(
        retirement.children.len(),
        1,
        "one descendant source is closure evidence"
    );
    assert_eq!(
        retirement.realizations.len(),
        2,
        "both descendant publications retire their realization"
    );
    let envelopes = checkpoint::envelopes(&checkpoint::capture(&app));
    for observed in seen.iter() {
        let native = envelopes
            .iter()
            .find(|e| e["commit"]["commit_id"].as_u64() == Some(observed.publication.0))
            .unwrap();
        let actual = native["required_output_origin"]["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                (
                    m["role"].as_str().unwrap().to_owned(),
                    m["posture"].clone(),
                    m["id"].clone(),
                )
            })
            .collect::<Vec<_>>();
        let expected = observed
            .roles
            .iter()
            .map(|(role, posture, id)| {
                (
                    role.clone(),
                    serde_json::Value::String(
                        match posture {
                            WorthQueryApplicationOutputPosture::Create => "Create",
                            WorthQueryApplicationOutputPosture::Preserve => "Preserve",
                            WorthQueryApplicationOutputPosture::Retire => "Retire",
                        }
                        .to_owned(),
                    ),
                    serde_json::to_value(id).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let mut actual = actual;
        let mut expected = expected;
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            actual, expected,
            "the public projection names exactly this publication's canonical roles"
        );
    }
    assert_absent(&request, &["root.1", "root.1.0"]);
    assert_live(&request, &["root", "root.0", "root.0.0"]);
}

#[test]
fn an_owned_leaf_record_reaches_its_publication_without_ancestor_or_sibling_publications() {
    let app = world::install(2, Selections::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let settled = settle(&app, &request);
    let (_, leaf) = settled
        .settlement()
        .outputs_for::<Schema, program::SplitChildSplitLeaf>()
        .find(|(demand, _)| demand.0 == "root.1.0")
        .expect("real leaf producer");
    let record = if let Ok(roles) = leaf.outputs_of::<producers::leaf::preserve::Outputs>() {
        roles
            .entity::<producers::leaf::preserve::Output>()
            .unwrap()
            .entity_id()
    } else {
        leaf.outputs_of::<producers::leaf::initial::Outputs>()
            .unwrap()
            .entity::<producers::leaf::initial::Output>()
            .unwrap()
            .entity_id()
    };
    close(settled, &request);
    let owner = request
        .query(Read("root.1.0".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let reached = app
        .runtime()
        .current_ownership_sources_for_record_for_test(record, &scope)
        .unwrap();
    assert_eq!(
        reached,
        [owner],
        "a record's publication scope is not an instruction to ascend the ownership tree"
    );
}
