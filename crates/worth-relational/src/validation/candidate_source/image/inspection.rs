//! Explicit diagnostic traversal of material retained by one Native image.
use super::*;

/// Visits retained material only; grants neither Source READ nor publication authority.
/// This scans every Source association in this image and is never an ordinary read.
pub fn inspect_managed_source_material(
    image: &ManagedIndexBranchImage,
    visitor: &mut dyn FnMut(&Program, &(dyn Any + Send + Sync)),
) {
    for (_, root) in image.directory().iter() {
        if let Some(payload) = root.payload() {
            visitor(&root.association.program, payload.as_ref());
        }
    }
}
