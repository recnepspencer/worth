//! Atomic history preserves every route's provenance and rejects altered closure anchors.
use super::{checkpoint::*, settlement::*, *};
#[test]
fn checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator() {
    let app = world::install(2, Arc::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle(&app, &request), &request);
    let before = capture(&app);
    resize(&app, &request, 1, 1);
    close(settle(&app, &request), &request);
    let after = capture(&app);
    assert_shrink_delta(&app, &before, &after);
    let history = envelopes(&after);
    let grouped = history
        .iter()
        .filter(|e| {
            e.get("producer_cascade_retirement")
                .is_some_and(|e| !e.is_null())
        })
        .collect::<Vec<_>>();
    assert_eq!(grouped.len(), 1, "one atomic generator succession");
    let event = grouped[0];
    assert!(event["required_output_origin"]["members"]
        .as_array()
        .unwrap()
        .iter()
        .all(|m| m["posture"] != "Create"));
    let group = &event["producer_cascade_retirement"];
    assert_eq!(group["anchors"].as_array().unwrap().len(), 1);
    let publications = group["publications"].as_array().unwrap();
    assert_eq!(
        publications.len(),
        2,
        "child and leaf routes retire separately"
    );
    for retired in publications {
        let predecessor = &retired["publication"]["ProducerRetirement"]["predecessor"];
        let prior = history
            .iter()
            .filter_map(|e| e.get("required_output_origin"))
            .find(|p| {
                p["commit"]["commit_id"] == *predecessor && p["meaning"] == retired["meaning"]
            })
            .expect("genuine predecessor route");
        for key in [
            "producer",
            "dependency_identity",
            "meaning",
            "birth",
            "birth_producer",
            "birth_dependency",
        ] {
            assert_eq!(retired[key], prior[key], "{key}");
        }
        assert_eq!(retired["commit"], event["commit"]);
        assert!(retired["members"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["posture"] == "Retire"));
        assert!(retired["relations"].as_array().unwrap().is_empty());
    }
    drop((principal, scope, app));
    let recovered = world::restore(after, Arc::default());
    let (scope, principal) = world::authenticate(&recovered);
    let request = recovered.runtime().request(&principal, &scope);
    close(settle(&recovered, &request), &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    resize(&recovered, &request, 2, 2);
    close(settle(&recovered, &request), &request);
    assert_live(
        &request,
        &["root", "root.0", "root.0.0", "root.1", "root.1.0"],
    );
}
#[test]
fn a_tampered_producer_closure_is_refused_after_query_checksum_admission() {
    let app = world::install(2, Arc::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    close(settle(&app, &request), &request);
    resize(&app, &request, 1, 1);
    close(settle(&app, &request), &request);
    let checkpoint = capture(&app);
    let tampered = tamper_anchor(&checkpoint);
    // Same frame size and valid Query checksum; the changed Native group must refuse.
    assert_eq!(checkpoint.bytes().len(), tampered.bytes().len());
    require_corruption_refusal(tampered);
    let omitted_leaf = omit_leaf_publication(&checkpoint);
    assert!(envelopes(&omitted_leaf)
        .iter()
        .any(|e| e["producer_cascade_retirement"]["publications"]
            .as_array()
            .is_some_and(|p| p.len() == 1)));
    require_corruption_refusal(omitted_leaf);
    let omitted_group = omit_retirement_group(&checkpoint);
    assert!(envelopes(&omitted_group).iter().all(|e| e
        .get("producer_cascade_retirement")
        .is_none_or(|g| g.is_null())));
    require_corruption_refusal(omitted_group);
}
fn require_corruption_refusal(tampered: installation::WorthQueryApplicationCheckpoint) {
    let restored = world::try_restore(tampered);
    match restored {
        Err(denial) => assert!(!format!("{denial:?}").contains("checksum"), "{denial:?}"),
        Ok(recovered) => {
            let (scope, principal) = world::authenticate(&recovered);
            let request = recovered.runtime().request(&principal, &scope);
            let mut operation = request
                .demand(producers::parent::Demand("root".into()))
                .retain_program_outputs::<Program, program::Tree>(&recovered)
                .unwrap_or_else(|e| panic!("{:?}", e.denial()));
            for _ in 0..128 {
                match operation.advance(&request) {
                    WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
                    WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                        panic!("tampered closure recovered as current")
                    }
                    WorthQueryProgramOutputOperationAdvance::Refused { .. }
                    | WorthQueryProgramOutputOperationAdvance::Terminal(_) => return,
                }
            }
            panic!("tampered closure did not terminate");
        }
    }
}
