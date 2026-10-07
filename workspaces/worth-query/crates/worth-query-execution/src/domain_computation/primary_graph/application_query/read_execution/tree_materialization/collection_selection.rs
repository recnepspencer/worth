//! Declared collection selection and its bounded page progress.

use super::*;

pub(in crate::domain_computation::primary_graph::application_query::read_execution) struct OrderedCollectionWindow
{
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) snapshot:
        worth_relational::facade::snapshots::SnapshotHandle,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) collection_path:
        String,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) index_id:
        DerivedIndexId,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) expected_generation:
        Option<DerivedIndexGenerationId>,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) after:
        Option<RelatedEntityOrderingBoundary>,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) page_width:
        usize,
}

pub(in crate::domain_computation::primary_graph::application_query::read_execution) struct TargetedCollectionChild
{
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) collection_path:
        String,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) child_entity_id:
        EntityId,
}

pub(in crate::domain_computation::primary_graph::application_query::read_execution) enum ResultTreeCollectionSelection
{
    Complete,
    Ordered(OrderedCollectionWindow),
    Targeted(TargetedCollectionChild),
}

pub(in crate::domain_computation::primary_graph::application_query::read_execution) struct OrderedCollectionProgress
{
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) generation_id:
        DerivedIndexGenerationId,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) next_boundary:
        Option<RelatedEntityOrderingBoundary>,
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) has_more:
        bool,
}

pub(in crate::domain_computation::primary_graph::application_query::read_execution) enum ActiveResultTreeCollectionSelection
{
    Complete,
    Ordered(ActiveOrderedCollectionWindow),
    Targeted(TargetedCollectionChild),
}

impl ActiveResultTreeCollectionSelection {
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn new(
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

    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn into_progress(
        self,
    ) -> Option<OrderedCollectionProgress> {
        match self {
            Self::Ordered(window) => window.into_progress(),
            Self::Complete | Self::Targeted(_) => None,
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_query::read_execution) struct ActiveOrderedCollectionWindow
{
    pub(super) request: Option<OrderedCollectionWindow>,
    pub(super) progress: Option<OrderedCollectionProgress>,
}

impl ActiveOrderedCollectionWindow {
    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn new(
        request: OrderedCollectionWindow,
    ) -> Self {
        Self {
            request: Some(request),
            progress: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query::read_execution) fn into_progress(
        self,
    ) -> Option<OrderedCollectionProgress> {
        self.progress
    }
}
