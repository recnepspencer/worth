//! One real installed graph is reused by every related route proof.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{application_entry::*, application_installation as installation};
pub(super) fn successor(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
) -> successor::host::Host {
    successor::output::reset_decodes();
    installation::in_memory_rostered_program_from_checkpoint_with_transition(
        successor::host::program(),
        installation::WorthQueryApplicationProgramRoster::new(),
        successor::schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        successor::host::limits(),
        checkpoint,
        installation::WorthQueryCheckpointProgramPredecessor::new(
            &host::program().revision().to_string(),
        )
        .unwrap(),
        installation::WorthQueryCheckpointTransitionResources::bounded(8192, 32, 65_536).unwrap(),
        installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        |_, _| Ok(()),
    )
    .expect("the real complete native output closure admits the target")
}
pub(super) fn edited() -> model::NestedValue {
    let mut value = model::authored_value();
    value.title = "regenerated γ".into();
    value.children[0].rank = 40;
    value.children[1].rank = -3;
    value.children[0].tag.0 = "updated-left".into();
    value
}
pub(super) fn heavy() -> model::NestedValue {
    let mut value = model::authored_value();
    value.children = (0..16)
        .map(|index| {
            let mut child = value.children[index % 2].clone();
            child.tag.0 = format!("{index:02}{}", "β".repeat(127));
            child
        })
        .collect();
    value
}
pub(super) type Page =
    worth_query_host::facade::primary_graph::WorthQueryApplicationContinuationPageResult<
        schema::CanonicalSchema,
        page_query::PageQuery,
        query::Parameters,
        page_query::PageRow,
        schema::Derived,
    >;
pub(super) fn page(host: &host::Host) -> Page {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .page(
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap(),
        )
        .unwrap()
}
pub(super) fn assert_old(page: &Page, child: &str) {
    assert_eq!(page.rows().len(), 1);
    assert_eq!(page.rows()[0].children.len(), 1);
    assert_eq!(page.rows()[0].children[0].0, child);
    let value = &page.rows()[0].children[0].1;
    assert_eq!(
        output::literals(value),
        (
            "ASSEMBLY Β",
            12,
            ["left".to_owned(), "right".to_owned()].as_slice()
        )
    );
}
pub(super) fn assert_target_pages(host: &successor::host::Host, edited: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world());
    let first = request
        .query(successor::page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .page(
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap(),
        )
        .unwrap();
    let generation = first
        .receipt()
        .ordered_index_generation()
        .expect("the actual installed related index served page one");
    let expected = if edited {
        (
            "V2 REGENERATED Γ",
            1037,
            vec!["v2:updated-left".to_owned(), "v2:right".to_owned()],
        )
    } else {
        (
            "V2 ASSEMBLY Β",
            1012,
            vec!["v2:left".to_owned(), "v2:right".to_owned()],
        )
    };
    assert_eq!(first.rows()[0].children[0].0, "derived-left");
    assert_eq!(
        successor::output::literals(&first.rows()[0].children[0].1),
        (expected.0, expected.1, expected.2.as_slice())
    );
    let token = first
        .into_parts()
        .1
        .expect("two genuine child relations require a token");
    let resume_scope = request_scope();
    let resume_principal = authenticate_operator(host.installed_schema(), &resume_scope);
    let resume_request = host
        .runtime()
        .request(&resume_principal, &resume_scope)
        .on_branch(host.runtime().current_world());
    let second = resume_request
        .query(successor::page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .resume(
            token,
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap(),
        )
        .unwrap();
    assert_eq!(
        second.receipt().ordered_index_generation(),
        Some(generation)
    );
    assert!(second.continuation().is_none());
    assert_eq!(second.rows()[0].children[0].0, "derived-right");
    assert_eq!(
        successor::output::literals(&second.rows()[0].children[0].1),
        (expected.0, expected.1, expected.2.as_slice())
    );
}

/// A new genuine current page observes all three-role Preserve changes.
pub(super) fn assert_original_current(host: &host::Host) {
    let first = page(host);
    assert_eq!(first.rows()[0].children[0].0, "derived-left");
    assert_eq!(output::literals(&first.rows()[0].children[0].1).1, 37);
    let token = first.into_parts().1.unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let second = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(page_query::ReadPage {
            identity: "derived-report".into(),
        })
        .resume(
            token,
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap(),
        )
        .unwrap();
    assert_eq!(second.rows()[0].children[0].0, "derived-right");
    assert_eq!(output::literals(&second.rows()[0].children[0].1).1, 37);
    assert!(second.continuation().is_none());
}
