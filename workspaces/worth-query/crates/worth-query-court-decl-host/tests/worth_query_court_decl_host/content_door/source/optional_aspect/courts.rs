//! Every optional-parent transition is observable through actual provider updates.
use super::*;
fn edit(host: &world::Host, create: bool, values: Option<Vec<i64>>, id: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(operation::Edit {
            key: "optional-row".into(),
            create,
            samples: values,
        })
        .without_source()
        .idempotency(&id.to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
fn source_and_index(host: &world::Host) -> Vec<query::Row> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let source = request
        .query(query::ReadSource {
            root: "record".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    let indexed = request
        .query(query::Find {
            root: "record".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    assert_eq!(
        indexed, source,
        "maintained index agrees with the complete Native Source"
    );
    source
}
#[test]
fn optional_aspect_content_appearance_replacement_and_removal_reach_source_update() {
    let calls = Arc::new(Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    edit(&host, true, None, "absent-row");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let find = || query::Find {
        root: "record".into(),
    };
    let index = request.query(find()).open_index::<Index>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    let absent = source_and_index(&host);
    assert_eq!(
        absent,
        [query::Row {
            key: "optional-row".into(),
            content: None
        }]
    );
    for (values, id) in [
        (Some(vec![7]), "present"),
        (Some(vec![23]), "replaced"),
        (None, "removed"),
    ] {
        let before = source_and_index(&host);
        calls.optional_changes.lock().unwrap().clear();
        let updates = calls.optional_updates.load(Ordering::SeqCst);
        edit(&host, false, values.clone(), id);
        assert!(
            calls.optional_updates.load(Ordering::SeqCst) > updates,
            "{id} must reach provider update"
        );
        let after = source_and_index(&host);
        assert_eq!(before[0].key, after[0].key);
        assert_ne!(before[0].content, after[0].content);
        assert_eq!(after[0].content.is_some(), values.is_some());
        assert!(
            calls
                .optional_changes
                .lock()
                .unwrap()
                .contains(&(Some(before[0].clone()), Some(after[0].clone()))),
            "provider changed slots agree with complete Source rows"
        );
    }
    index.close();
}
#[test]
fn optional_aspect_removal_is_a_source_change_when_every_other_projected_field_is_equal() {
    let calls = Arc::new(Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    edit(&host, true, None, "twin-absent");
    edit(&host, false, Some(vec![7]), "twin-present");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let find = || query::Find {
        root: "record".into(),
    };
    let index = request.query(find()).open_index::<Index>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    let before = source_and_index(&host);
    calls.optional_changes.lock().unwrap().clear();
    let updates = calls.optional_updates.load(Ordering::SeqCst);
    edit(&host, false, None, "twin-remove");
    let after = source_and_index(&host);
    assert_eq!(before[0].key, after[0].key);
    assert!(before[0].content.is_some());
    assert_eq!(after[0].content, None);
    assert!(calls.optional_updates.load(Ordering::SeqCst) > updates);
    assert!(calls
        .optional_changes
        .lock()
        .unwrap()
        .contains(&(Some(before[0].clone()), Some(after[0].clone()))));
    index.close();
}
