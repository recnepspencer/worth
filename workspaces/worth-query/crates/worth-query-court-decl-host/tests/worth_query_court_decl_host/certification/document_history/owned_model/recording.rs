use super::*;
type Action = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    edit::member::Binding,
    Completion,
>;

#[test]
fn stored_direct_and_transitive_ownership_admit_child_scoped_history_edits() {
    let host = world::install();
    let mut document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    for (key, expected) in [("direct", 101), ("nested", 202)] {
        let old_branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        let old_value = world::member_value(&host, old_branch, key);
        let mut edit = document
            .edit(edit::member::Intent(edit::Edit {
                key: key.into(),
                value: expected,
            }))
            .idempotency_owned(&request, key.to_owned())
            .unwrap()
            .retain_in_program::<Program, Action>(&host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        loop {
            edit = match edit.advance(&request) {
                WorthQueryHistoryEditAdvance::Progress(next) => {
                    assert_eq!(world::member_value(&host, old_branch, key), old_value);
                    next
                }
                WorthQueryHistoryEditAdvance::Ready(next) => {
                    edit = next;
                    break;
                }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            };
        }
        let WorthQueryHistoryEditFinish {
            document: returned,
            outcome,
        } = edit
            .finish(&request)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        let WorthQueryHistoryEditOutcome::Recorded { step, .. } = outcome else {
            panic!("not recorded")
        };
        assert_eq!(step.label(), Some("Edit member"));
        document = returned;
        assert_eq!(
            world::member_value(
                &host,
                document
                    .selection(&request)
                    .unwrap()
                    .physical_endpoint()
                    .product_branch(),
                key
            ),
            expected
        );
        assert_eq!(
            world::member_value(
                &host,
                document
                    .selection(&request)
                    .unwrap()
                    .physical_endpoint()
                    .product_branch(),
                "foreign"
            ),
            33
        );
    }
    assert_ne!(document.current_step(&request).unwrap().id(), root.id());
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn foreign_members_and_undeclared_reference_links_refuse_before_history_effects() {
    for key in ["foreign", "reference"] {
        let host = world::install();
        let document = world::attach(&host);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let step_before = document.current_step(&request).unwrap().id();
        let branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        let original = world::member_value(&host, branch, key);
        let edit = document
            .edit(edit::member::Intent(edit::Edit {
                key: key.into(),
                value: 999,
            }))
            .idempotency_owned(&request, key.to_owned())
            .unwrap()
            .retain_in_program::<Program, Action>(&host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        let WorthQueryHistoryEditAdvance::Refused { edit, denial } = edit.advance(&request) else {
            panic!("ownership must refuse")
        };
        assert!(
            matches!(
                denial,
                WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::UnownedOperationScope { .. }
                ))
            ),
            "{denial:?}"
        );
        let WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::UnownedOperationScope { subject, .. },
        )) = &denial
        else {
            unreachable!()
        };
        assert_eq!(
            (
                subject.partition_id.as_u32(),
                subject.local_slot_value(),
                subject.generation_value()
            ),
            world::member_identity(&host, branch, key)
        );
        assert_eq!(world::member_value(&host, branch, key), original);
        let document = edit
            .discard(&request)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        assert_eq!(document.current_step(&request).unwrap().id(), step_before);
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
    }
}
