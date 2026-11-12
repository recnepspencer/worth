//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryWorkflowDefinitionPublicationPreparationDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/workflow/definition/publication.rs", "WorthQueryWorkflowDefinitionPublicationPreparationDenial", 128 * 1024);
    }
}
