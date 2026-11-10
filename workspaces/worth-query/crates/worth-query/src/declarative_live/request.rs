use crate::authoring::{AspectFieldKey, OrderingDirection, TraversalSelector};
use crate::authorized_projection::AuthorizedDeclarativeProjection;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_evolution::{InspectorIdentityArtifact, InspectorIdentityClassification};
use crate::view_shape::ViewShapeDescriptor;
use worth_foundational::facade::AspectKey;

use super::predicates::{
    DeclarativeEqualityFilter, DeclarativeNativeComparisonFilter, DeclarativePredicateFilter,
    DeclarativePresenceFilter, DeclarativeSetMembershipFilter, DeclarativeStringContainsFilter,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarativeProjectionField {
    source: AspectFieldKey,
    delivered_name: String,
}

impl DeclarativeProjectionField {
    pub fn new(source: AspectFieldKey) -> Self {
        let delivered_name = source.field().as_str().to_string();
        Self {
            delivered_name,
            source,
        }
    }

    pub(crate) fn from_authoring_parts(
        aspect: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self::new(
            AspectFieldKey::from_authoring_parts(aspect, field)
                .expect("declarative projection fields require non-empty aspect and field names"),
        )
    }

    pub fn delivered_as(mut self, delivered_name: impl Into<String>) -> Self {
        self.delivered_name = delivered_name.into();
        self
    }

    pub fn source_field_key(&self) -> &AspectFieldKey {
        &self.source
    }

    pub fn delivered_name(&self) -> &str {
        &self.delivered_name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarativeOrderingField {
    source: AspectFieldKey,
    direction: OrderingDirection,
}

impl DeclarativeOrderingField {
    pub fn ascending(source: AspectFieldKey) -> Self {
        Self {
            source,
            direction: OrderingDirection::Ascending,
        }
    }

    pub fn descending(source: AspectFieldKey) -> Self {
        Self {
            source,
            direction: OrderingDirection::Descending,
        }
    }

    pub fn source_field_key(&self) -> &AspectFieldKey {
        &self.source
    }

    pub fn direction(&self) -> OrderingDirection {
        self.direction
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarativeLiveViewShape {
    ListSplice,
    Table,
    Detail,
    InspectorObserved,
    InspectorFocused {
        focused_aspect: AspectKey,
    },
    IdentityAwareInspectorFocused {
        focused_aspect: AspectKey,
        classification: InspectorIdentityClassification,
    },
    KanbanGrouped {
        grouping_aspect: AspectKey,
    },
}

impl DeclarativeLiveViewShape {
    pub fn list_splice() -> Self {
        Self::ListSplice
    }

    pub fn table() -> Self {
        Self::Table
    }

    pub fn detail() -> Self {
        Self::Detail
    }

    pub fn inspector_observed() -> Self {
        Self::InspectorObserved
    }

    pub fn inspector_focused(focused_aspect: AspectKey) -> Self {
        Self::InspectorFocused { focused_aspect }
    }

    pub fn identity_aware_inspector_focused(
        focused_aspect: AspectKey,
        classification: InspectorIdentityClassification,
    ) -> Self {
        Self::IdentityAwareInspectorFocused {
            focused_aspect,
            classification,
        }
    }

    pub fn kanban_grouped(grouping_aspect: AspectKey) -> Self {
        Self::KanbanGrouped { grouping_aspect }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ListSplice => "list_splice",
            Self::Table => "table",
            Self::Detail => "detail",
            Self::InspectorObserved => "inspector_observed",
            Self::InspectorFocused { .. } => "inspector_focused",
            Self::IdentityAwareInspectorFocused { .. } => "identity_aware_inspector_focused",
            Self::KanbanGrouped { .. } => "kanban_grouped",
        }
    }

    pub(super) fn collection_backed(&self) -> bool {
        matches!(
            self,
            Self::ListSplice | Self::Table | Self::KanbanGrouped { .. }
        )
    }

    pub(super) fn view_shape_descriptor(&self) -> ViewShapeDescriptor {
        match self {
            Self::ListSplice | Self::Table => ViewShapeDescriptor::table(),
            Self::Detail => ViewShapeDescriptor::detail(),
            Self::InspectorObserved => ViewShapeDescriptor::inspector_detail_observed(),
            Self::InspectorFocused { focused_aspect } => {
                ViewShapeDescriptor::inspector_detail_focused(focused_aspect.clone())
            }
            Self::IdentityAwareInspectorFocused {
                focused_aspect,
                classification,
            } => ViewShapeDescriptor::identity_aware_inspector_detail_focused(
                focused_aspect.clone(),
                *classification,
            ),
            Self::KanbanGrouped { grouping_aspect } => {
                ViewShapeDescriptor::kanban_grouped(grouping_aspect.clone())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarativeLiveQueryRequest {
    target: String,
    view_shape: DeclarativeLiveViewShape,
    query_projection: Vec<DeclarativeProjectionField>,
    result_fields: Vec<DeclarativeProjectionField>,
    predicate_filters: Vec<DeclarativePredicateFilter>,
    traversal: Vec<TraversalSelector>,
    ordering: Vec<DeclarativeOrderingField>,
    inspector_identity: Option<InspectorIdentityArtifact>,
}

impl DeclarativeLiveQueryRequest {
    pub fn new(target: impl Into<String>, view_shape: DeclarativeLiveViewShape) -> Self {
        Self {
            target: target.into(),
            view_shape,
            query_projection: Vec::new(),
            result_fields: Vec::new(),
            predicate_filters: Vec::new(),
            traversal: Vec::new(),
            ordering: Vec::new(),
            inspector_identity: None,
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn target_collection_identity(
        &self,
    ) -> Result<crate::runtime::WorthQueryMutationTargetCollectionIdentity, IdentityDerivationDenial>
    {
        crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
            "declarative-live-request-target",
            self.target.clone(),
        )
    }

    pub fn view_shape(&self) -> &DeclarativeLiveViewShape {
        &self.view_shape
    }

    pub fn query_projection(&self) -> &[DeclarativeProjectionField] {
        &self.query_projection
    }

    pub fn projection(&self) -> &[DeclarativeProjectionField] {
        self.result_fields()
    }

    pub fn result_fields(&self) -> &[DeclarativeProjectionField] {
        &self.result_fields
    }

    pub fn predicate_filters(&self) -> &[DeclarativePredicateFilter] {
        &self.predicate_filters
    }

    pub fn traversal(&self) -> &[TraversalSelector] {
        &self.traversal
    }

    pub fn ordering(&self) -> &[DeclarativeOrderingField] {
        &self.ordering
    }

    pub fn inspector_identity(&self) -> Option<&InspectorIdentityArtifact> {
        self.inspector_identity.as_ref()
    }

    pub fn project(mut self, field: DeclarativeProjectionField) -> Self {
        self.query_projection.push(field.clone());
        self.result_fields.push(field);
        self
    }

    pub fn where_equal(mut self, filter: DeclarativeEqualityFilter) -> Self {
        self.predicate_filters
            .push(DeclarativePredicateFilter::Equality(filter));
        self
    }

    pub fn where_greater_than(mut self, filter: DeclarativeNativeComparisonFilter) -> Self {
        self.predicate_filters
            .push(DeclarativePredicateFilter::NativeComparison(filter));
        self
    }

    pub fn where_less_than(mut self, filter: DeclarativeNativeComparisonFilter) -> Self {
        self.predicate_filters
            .push(DeclarativePredicateFilter::NativeComparison(filter));
        self
    }

    pub fn where_contains(mut self, filter: DeclarativeStringContainsFilter) -> Self {
        self.predicate_filters
            .push(DeclarativePredicateFilter::StringContains(filter));
        self
    }

    pub fn where_in(mut self, filter: DeclarativeSetMembershipFilter) -> Self {
        self.predicate_filters
            .push(DeclarativePredicateFilter::SetMembership(filter));
        self
    }

    pub fn where_present(mut self, filter: DeclarativePresenceFilter) -> Self {
        self.predicate_filters
            .push(DeclarativePredicateFilter::Presence(filter));
        self
    }

    pub fn traverse(mut self, selector: TraversalSelector) -> Self {
        self.traversal.push(selector);
        self
    }

    pub fn order_by(mut self, field: DeclarativeProjectionField) -> Self {
        self.ordering.push(DeclarativeOrderingField::ascending(
            field.source_field_key().clone(),
        ));
        self
    }

    pub fn order_by_direction(mut self, field: DeclarativeOrderingField) -> Self {
        self.ordering.push(field);
        self
    }

    pub fn with_inspector_identity(mut self, artifact: InspectorIdentityArtifact) -> Self {
        self.inspector_identity = Some(artifact);
        self
    }

    pub(crate) fn project_query_only(mut self, field: DeclarativeProjectionField) -> Self {
        self.query_projection.push(field);
        self
    }

    pub(crate) fn result_field(mut self, field: DeclarativeProjectionField) -> Self {
        self.result_fields.push(field);
        self
    }

    pub(crate) fn with_authorized_query_projection(
        mut self,
        authorized: AuthorizedDeclarativeProjection,
    ) -> Self {
        self.query_projection = authorized.into_fields();
        self
    }
}
