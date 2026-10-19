//! Independent-site meaning participates in program/evolution identity as well as schema identity.
use super::{base, framed_record, ApplicationOperationBodyProgram};
use crate::application_schema::ApplicationOperationBodyCreation as Creation;
pub(super) fn append(body: &ApplicationOperationBodyProgram, record: String) -> String {
    let Some(root) = body
        .creation()
        .filter(|c| !c.peers().is_empty() || !c.path_links().is_empty())
    else {
        return record;
    };
    framed_record(
        "operation-body-independent-sites-v1",
        std::iter::once(("body", record))
            .chain(
                root.path_links()
                    .iter()
                    .map(|p| ("root-path-link", p.canonical_record())),
            )
            .chain(root.peers().iter().map(|p| ("peer", site_record(p)))),
    )
}
fn site_record(site: &Creation) -> String {
    framed_record(
        "independent-creation-v1",
        std::iter::once(("creation", base::creation_record(site)))
            .chain(site.links().iter().map(|l| ("link", l.canonical_record())))
            .chain(
                site.path_links()
                    .iter()
                    .map(|p| ("path-link", p.canonical_record())),
            )
            .chain(
                site.children()
                    .iter()
                    .map(|child| ("child", super::child_record(child))),
            ),
    )
}
