//! Adoption changes Query support, never Native values or allocation coordinates.
use super::super::failure;
use super::*;

pub(super) fn assert_unchanged(host: &world::DirHost<program::Lazy>, before: &[u8]) {
    let after = failure::checkpoint(host);
    worth_query_testkit_document_retention::facade::directory_home::assert_native_checkpoint_fully_unchanged(
        before, &after,
    );
}
