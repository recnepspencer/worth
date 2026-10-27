use crate::domain_operation::{WorthQueryOperationGraphReadScope, WorthQueryOperationTouchScope};

/// Comparison-ready installed read/touch index. It retains typed scopes and
/// never reparses declaration or canonical material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationReadTouchOverlapIndex {
    reads: Vec<WorthQueryOperationGraphReadScope>,
    touches: Vec<WorthQueryOperationTouchScope>,
    touch_overlap_counts: Vec<usize>,
}

impl WorthQueryOperationReadTouchOverlapIndex {
    pub(crate) fn new(
        reads: Vec<WorthQueryOperationGraphReadScope>,
        touches: Vec<WorthQueryOperationTouchScope>,
    ) -> Self {
        let touch_overlap_counts = touches
            .iter()
            .map(|touch| {
                reads
                    .iter()
                    .filter(|read| read_intersects_touch(read, touch))
                    .count()
            })
            .collect();
        Self {
            reads,
            touches,
            touch_overlap_counts,
        }
    }

    pub fn reads(&self) -> &[WorthQueryOperationGraphReadScope] {
        &self.reads
    }

    pub fn touches(&self) -> &[WorthQueryOperationTouchScope] {
        &self.touches
    }

    pub fn intersects(
        &self,
        read: &WorthQueryOperationGraphReadScope,
        touch: &WorthQueryOperationTouchScope,
    ) -> bool {
        read_intersects_touch(read, touch)
    }

    /// Number of installed read scopes intersecting one indexed touch scope.
    /// Installation pays the cross-product once; execution performs O(1)
    /// overlap lookup after resolving the selected touch index.
    pub fn touch_overlap_count(&self, touch_index: usize) -> Option<usize> {
        self.touch_overlap_counts.get(touch_index).copied()
    }
}

fn read_intersects_touch(
    read: &WorthQueryOperationGraphReadScope,
    touch: &WorthQueryOperationTouchScope,
) -> bool {
    match (read, touch) {
        (
            WorthQueryOperationGraphReadScope::Relation(read),
            WorthQueryOperationTouchScope::InitializeRelationField(touch),
        ) => {
            let relation = touch.relation();
            read.schema() == relation.schema()
                && read.relation() == relation.relation()
                && read.from() == relation.from()
                && read.to() == relation.to()
        }
        (
            WorthQueryOperationGraphReadScope::Entity(read),
            WorthQueryOperationTouchScope::CreateEntity(touch)
            | WorthQueryOperationTouchScope::DeleteEntity(touch),
        ) => read.schema() == touch.schema() && read.semantic_key() == touch.entity(),
        (
            WorthQueryOperationGraphReadScope::NativeProjection(read),
            WorthQueryOperationTouchScope::WriteField(touch),
        ) => {
            read.schema() == touch.schema()
                && read.entity().semantic_key() == touch.entity()
                && read.projection().contract().identity() == touch.contract().identity()
                && read.projection().contract().revision() == touch.contract().revision()
                && (read.projection().mask().is_whole_aspect()
                    || read
                        .projection()
                        .mask()
                        .paths()
                        .contains(touch.field_path()))
        }
        (
            WorthQueryOperationGraphReadScope::Relation(read),
            WorthQueryOperationTouchScope::LinkRelation(touch)
            | WorthQueryOperationTouchScope::UnlinkRelation(touch),
        ) => {
            read.schema() == touch.schema()
                && read.relation() == touch.relation()
                && read.from() == touch.from()
                && read.to() == touch.to()
        }
        _ => false,
    }
}
