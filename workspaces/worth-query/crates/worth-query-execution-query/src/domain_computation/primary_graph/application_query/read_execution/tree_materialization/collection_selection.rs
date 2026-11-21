//! Declared collection selection and its bounded page progress.

use super::*;

pub struct OrderedCollectionWindow
{
    pub snapshot:
        worth_relational::facade::snapshots::SnapshotHandle,
    pub collection_path:
        String,
    pub index_id:
        DerivedIndexId,
    pub expected_generation:
        Option<DerivedIndexGenerationId>,
    pub after:
        Option<RelatedEntityOrderingBoundary>,
    pub page_width:
        usize,
}

pub struct TargetedCollectionChild
{
    pub collection_path:
        String,
    pub child_entity_id:
        EntityId,
}

pub enum ResultTreeCollectionSelection
{
    Complete,
    Ordered(OrderedCollectionWindow),
    Targeted(TargetedCollectionChild),
}

pub struct OrderedCollectionProgress
{
    pub generation_id:
        DerivedIndexGenerationId,
    pub next_boundary:
        Option<RelatedEntityOrderingBoundary>,
    pub has_more:
        bool,
}

pub enum ActiveResultTreeCollectionSelection
{
    Complete,
    Ordered(ActiveOrderedCollectionWindow),
    Targeted(TargetedCollectionChild),
}

impl ActiveResultTreeCollectionSelection {
    pub fn new(
        selection: ResultTreeCollectionSelection,
    ) -> Self {
        match selection {
            ResultTreeCollectionSelection::Complete => Self::Complete,
            ResultTreeCollectionSelection::Ordered(window) => {
                Self::Ordered(ActiveOrderedCollectionWindow::new(window))
            }
            ResultTreeCollectionSelection::Targeted(target) => Self::Targeted(target),
        }
    }

    pub fn into_progress(
        self,
    ) -> Option<OrderedCollectionProgress> {
        match self {
            Self::Ordered(window) => window.into_progress(),
            Self::Complete | Self::Targeted(_) => None,
        }
    }
}

pub struct ActiveOrderedCollectionWindow
{
    pub request: Option<OrderedCollectionWindow>,
    pub progress: Option<OrderedCollectionProgress>,
}

impl ActiveOrderedCollectionWindow {
    pub fn new(
        request: OrderedCollectionWindow,
    ) -> Self {
        Self {
            request: Some(request),
            progress: None,
        }
    }

    pub fn into_progress(
        self,
    ) -> Option<OrderedCollectionProgress> {
        self.progress
    }
}
