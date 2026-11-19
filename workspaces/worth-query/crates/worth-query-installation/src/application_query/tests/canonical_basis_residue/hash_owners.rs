//! Exact admitted hash owners; no directory-wide SHA exception.
use std::path::Path;

pub(super) fn admits(path: &Path, source: &str) -> bool {
    if path.ends_with(Path::new(
        "application_query/projection/projected_tree/content.rs",
    )) {
        assert!(source.contains("struct ContentWriter<'a, E>"));
        assert!(source.contains("(self.admit)(8)?"));
        assert!(source.contains("(self.admit)(chunk.len() as u64)?"));
        assert!(source.contains("write_aspect_value_identity_material(value, &mut scalar)?"));
        return true;
    }
    if path.ends_with(Path::new(
        "authorization/retained_capability_request/material.rs",
    )) {
        assert!(source.contains("worth-query:capability-use:v1"));
        assert!(source.contains("(self.admit)(8)?"));
        assert!(source.contains("(self.admit)(chunk.len() as u64)?"));
        return true;
    }
    if path.ends_with(Path::new("worth-relational/src/content/frame/integrity.rs")) {
        assert!(source.contains("worth.native.content.v1"));
        assert!(source.contains("control.checkpoint(DOMAIN.len() as u64)?"));
        assert!(source.contains("control.checkpoint(chunk.len() as u64)?"));
        assert!(source.contains("frame.chunks(4096)"));
        return true;
    }
    if path.ends_with(Path::new(
        "application_query/managed_index/source/projection_support/commitment.rs",
    )) {
        // The applied-source acquisition commitment charges every fact and
        // every text byte it absorbs before hashing it.
        assert!(source.contains("worth-query:applied-source-acquisition:v1"));
        assert!(source.contains("SourceFactsAllocation::work(execution, 96)?"));
        assert!(source.contains("length.checked_add(8).ok_or(Cause::WorkCounterOverflow)?"));
        return true;
    }
    false
}
