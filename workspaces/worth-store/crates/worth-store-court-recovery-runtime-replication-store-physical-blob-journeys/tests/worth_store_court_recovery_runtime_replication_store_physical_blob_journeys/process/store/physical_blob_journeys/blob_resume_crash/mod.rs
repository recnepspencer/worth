#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_resume_crash::*;

#[test]
fn killed_chunk_writer_resumes_selected_prefix_without_duplicate_claim() {
    resume_killed_prefix("crash-chunk");
}

#[test]
fn killed_frontier_writer_discards_partial_input_and_resumes_exact_selected_prefix() {
    resume_killed_prefix("crash-frontier-partial");
}
