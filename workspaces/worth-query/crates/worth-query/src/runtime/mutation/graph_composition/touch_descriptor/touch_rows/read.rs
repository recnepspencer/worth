use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryMutationFamily, WorthQueryMutationTargetCollectionIdentity,
};

use super::super::read_verb::WorthQueryGraphTouchReadVerb;
use super::{WorthQueryGraphTouchDescriptorRow, WorthQueryGraphTouchDescriptorRowInput};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryGraphReadTouchShape {
    aspect_touches: Vec<WorthQueryAspectTouch>,
}

impl WorthQueryGraphReadTouchShape {
    pub fn new(aspect_touches: impl IntoIterator<Item = WorthQueryAspectTouch>) -> Self {
        Self {
            aspect_touches: aspect_touches
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
        }
    }

    pub fn aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.aspect_touches
    }
}

pub(in super::super) fn derive_read_touch_rows(
    collection: &str,
    verbs: impl IntoIterator<Item = WorthQueryGraphTouchReadVerb>,
    shape: &WorthQueryGraphReadTouchShape,
) -> Result<Vec<WorthQueryGraphTouchDescriptorRow>, IdentityDerivationDenial> {
    verbs
        .into_iter()
        .enumerate()
        .map(|(component_index, verb)| read_touch_row(collection, component_index, verb, shape))
        .collect::<Result<Vec<_>, _>>()
}

fn read_touch_row(
    collection: &str,
    component_index: usize,
    verb: WorthQueryGraphTouchReadVerb,
    shape: &WorthQueryGraphReadTouchShape,
) -> Result<WorthQueryGraphTouchDescriptorRow, IdentityDerivationDenial> {
    WorthQueryGraphTouchDescriptorRow::new(WorthQueryGraphTouchDescriptorRowInput {
        component_index,
        mutation_family: WorthQueryMutationFamily::Assertion,
        read_verb: Some(verb),
        program_step_kind: None,
        lifecycle_family: None,
        declared_collection: Some(WorthQueryMutationTargetCollectionIdentity::new(
            "graph-read-touch-row",
            collection,
        )?),
        relation_kind_id: None,
        declared_symbol: None,
        declared_aspect_operations: Vec::new(),
        touched_aspects: shape.aspect_touches().to_vec(),
        has_symbolic_target_reference: false,
        has_existing_truth_binding: false,
        symbolic_aspect_reference_count: 0,
    })
}
