//! Runtime-length mixed actions use one original candidate and one history edge.
use super::*;
mod identity;
type ModelAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    edit::model::Binding,
    Completion,
>;
pub(super) type MemberAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    edit::member::Binding,
    Completion,
>;
type Gesture<'r> = WorthQueryGroupedHistoryDocument<
    'r,
    Schema,
    History,
    Program,
    WorthQueryHistoryGestureMembers<Schema, History, Program>,
>;
type Request<'a, 'b, 'c> = WorthQueryApplicationRequest<'a, 'b, 'c, Schema>;
fn model(value: u64) -> edit::model::Intent {
    edit::model::Intent(edit::Edit {
        key: "a".into(),
        value,
    })
}
fn member(value: u64) -> edit::member::Intent {
    edit::member::Intent(edit::Edit {
        key: "direct".into(),
        value,
    })
}
fn drive(
    group: &mut Gesture<'_>,
    request: &Request<'_, '_, '_>,
    stop: impl Fn(&Gesture<'_>) -> bool,
) {
    for _ in 0..128 {
        if stop(group) {
            return;
        }
        let (document, execution) = group.parts();
        execution.advance(document, request).unwrap();
    }
    panic!("finite mixed gesture did not reach its expected posture")
}
fn open<'r>(
    document: world::Document<'r>,
    host: &world::Host,
    request: &Request<'_, '_, '_>,
    key: u128,
) -> Gesture<'r> {
    document
        .gesture(host, request, key)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
}

#[test]
fn mixed_runtime_member_lists_publish_one_atomic_step_and_original_typed_results() {
    for count in [2, 3, 5] {
        let host = world::install();
        let document = world::attach(&host);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let root = document.current_step(&request).unwrap();
        let old_branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        let mut group = open(document, &host, &request, 54);
        let mut expected_model = 1;
        let mut expected_member = 11;
        // The list's length and each entry's concrete action are selected at runtime.
        for index in 0..count {
            let slot = if index % 2 == 0 {
                expected_model = 100 + index as u64;
                group.push::<_, ModelAction>(
                    &request,
                    model(expected_model),
                    format!("model:{index}"),
                )
            } else {
                expected_member = 200 + index as u64;
                group.push::<_, MemberAction>(
                    &request,
                    member(expected_member),
                    format!("member:{index}"),
                )
            }
            .unwrap();
            assert_eq!(slot, index);
        }
        for completed in 1..=count {
            drive(&mut group, &request, |group| {
                group.completed_members() == completed
            });
            assert_eq!(group.document().current_step(&request).unwrap(), root);
            assert_eq!(world::member_value(&host, old_branch, "direct"), 11);
            assert_eq!(
                host.request(&principal, &scope)
                    .on_branch(old_branch)
                    .query(query::model::Read("a".into()))
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                1
            );
        }
        assert!(matches!(
            group.push::<_, ModelAction>(&request, model(3), "late".into()),
            Err(WorthQueryHistoryGroupDenial::MembershipFrozen)
        ));
        let (document, execution) = group.parts();
        let step = execution.complete(document, &request).unwrap();
        assert_eq!(step.parent(), Some(root.id()));
        assert_ne!(step.id(), root.id());
        let finished = group
            .finish(&request)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        assert_eq!(finished.completions.len(), count);
        for (index, completion) in finished.completions.iter().enumerate() {
            assert_eq!(
                completion
                    .original::<edit::model::Intent, ModelAction>()
                    .is_some(),
                index % 2 == 0
            );
            assert_eq!(
                completion
                    .original::<edit::member::Intent, MemberAction>()
                    .is_some(),
                index % 2 != 0
            );
        }
        assert_eq!(
            finished.document.read(&request).unwrap().rows()[0].value,
            expected_model
        );
        let branch = finished
            .document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        assert_eq!(
            world::member_value(&host, branch, "direct"),
            expected_member
        );
        drop(finished.completions);
        host.with_host_advancement(|phase| finished.document.close(&phase).unwrap())
            .unwrap();
    }
}

#[test]
fn a_later_mixed_member_domain_denial_aborts_the_whole_private_gesture() {
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut group = open(document, &host, &request, 5401);
    group
        .push::<_, ModelAction>(&request, model(300), "good".into())
        .unwrap();
    group
        .push::<_, MemberAction>(&request, member(1001), "domain-denied".into())
        .unwrap();
    drive(&mut group, &request, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::Terminal
    });
    assert_eq!(group.completed_members(), 1);
    assert_eq!(group.document().current_step(&request).unwrap(), root);
    assert_eq!(group.document().read(&request).unwrap().rows()[0].value, 1);
    let document = group
        .abort(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(document.current_step(&request).unwrap(), root);
    assert_eq!(document.read(&request).unwrap().rows()[0].value, 1);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn an_empty_gesture_freezes_and_refuses_without_creating_a_candidate() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = open(world::attach(&host), &host, &request, 5402);
    let root = group.document().current_step(&request).unwrap();
    let (document, execution) = group.parts();
    assert!(matches!(
        execution.advance(document, &request),
        Err(WorthQueryHistoryGroupDenial::History(
            WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::EmptyGroup
            )
        ))
    ));
    assert!(matches!(
        group.push::<_, ModelAction>(&request, model(3), "late".into()),
        Err(WorthQueryHistoryGroupDenial::MembershipFrozen)
    ));
    let document = group
        .abort(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(document.current_step(&request).unwrap(), root);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
