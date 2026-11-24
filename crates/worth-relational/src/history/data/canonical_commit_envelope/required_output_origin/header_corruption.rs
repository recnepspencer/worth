//! Operate on independently copied canonical headers inside the history court.
use super::{CanonicalAuthoredRestoration, CanonicalAuthoredRetirement};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::CanonicalAuthoritativePatch;
use std::sync::Arc;

impl CanonicalCommitEnvelope {
    pub(in crate::history) fn edit_retirement_header_for_test(
        &mut self,
        edit: impl FnOnce(&mut CanonicalAuthoredRetirement),
    ) {
        let event = self
            .body
            .authored_cascade_retirement
            .as_ref()
            .expect("lawful cascade");
        let mut copy = CanonicalAuthoredRetirement {
            _decoded_memory: event._decoded_memory.clone(),
            anchor: event.anchor,
            owned_members: event.owned_members,
            birth: event.birth,
            co_anchors: event.co_anchors.clone(),
            publications: event.publications.clone(),
        };
        edit(&mut copy);
        self.body.authored_cascade_retirement = Some(Arc::new(copy));
    }

    pub(in crate::history) fn edit_restoration_header_for_test(
        &mut self,
        edit: impl FnOnce(&mut CanonicalAuthoredRestoration, &mut CanonicalAuthoritativePatch),
    ) {
        let event = self
            .body
            .authored_cascade_restoration
            .as_ref()
            .expect("lawful restoration");
        let mut copy = CanonicalAuthoredRestoration {
            _decoded_memory: event._decoded_memory.clone(),
            owned_members: event.owned_members,
            retirement: event.retirement,
            original_anchor: event.original_anchor,
            anchor: event.anchor,
            co_anchors: event.co_anchors.clone(),
            entities: event.entities.clone(),
            relations: event.relations.clone(),
            publications: event.publications.clone(),
        };
        edit(&mut copy, &mut self.body.patch);
        self.body.authored_cascade_restoration = Some(Arc::new(copy));
    }
}

// These are descriptive canonical headers, not the envelope's diagnostic or
// authority-bearing fields. Keep the enclosing genuine envelope intact.
fn copy_header<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) -> T {
    rmp_serde::from_slice(&rmp_serde::to_vec_named(value).expect("header encodes"))
        .expect("descriptive header decodes")
}
pub(in crate::history) fn origin_header_for_test(
    publications: &mut [Arc<crate::history::data::CanonicalRequiredOutputOrigin>],
    index: usize,
) -> &mut crate::history::data::CanonicalRequiredOutputOrigin {
    let original = &publications[index];
    let mut copied: crate::history::data::CanonicalRequiredOutputOrigin =
        copy_header(original.as_ref());
    // Carry the real issuer; serde never reconstructs this proof from a projection.
    copied.issuer = original.issuer.clone();
    publications[index] = Arc::new(copied);
    Arc::get_mut(&mut publications[index]).expect("owned corrupted header")
}
