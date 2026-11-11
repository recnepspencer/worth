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

    /// Dependency ordering must conservatively include unknown declared
    /// footprints and membership changes, rather than infer independence.
    pub fn may_intersect(
        &self,
        read: &WorthQueryOperationGraphReadScope,
        touch: &WorthQueryOperationTouchScope,
    ) -> bool {
        use WorthQueryOperationGraphReadScope as Read;
        use WorthQueryOperationTouchScope as Touch;
        match (read, touch) {
            (Read::DerivedSource(_) | Read::Components(_), _) | (_, Touch::DeclaredDomain(_)) => {
                true
            }
            (
                Read::NativeProjection(read),
                Touch::CreateEntity(write) | Touch::DeleteEntity(write),
            ) => read.schema() == write.schema() && read.entity().semantic_key() == write.entity(),
            (Read::Relation(read), Touch::DeleteEntity(write)) => {
                read.schema() == write.schema()
                    && (read.from() == write.entity() || read.to() == write.entity())
            }
            _ => self.intersects(read, touch),
        }
    }

    /// Query reads and graph reads share this installed overlap owner. No
    /// execution layer reconstructs field meaning or guesses record locality.
    pub fn query_intersects(
        &self,
        schema: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
        query: &crate::application_query::WorthQueryInstalledGraphReadContract,
        touch: &WorthQueryOperationTouchScope,
    ) -> bool {
        query_intersects_touch(schema, query, touch)
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
            WorthQueryOperationGraphReadScope::NativeProjection(read),
            WorthQueryOperationTouchScope::RemoveAspect(touch),
        ) => {
            read.schema() == touch.schema()
                && read.entity().semantic_key() == touch.entity()
                && read.projection().contract() == touch.contract()
        }
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

fn query_intersects_touch(
    schema: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
    query: &crate::application_query::WorthQueryInstalledGraphReadContract,
    touch: &WorthQueryOperationTouchScope,
) -> bool {
    use WorthQueryOperationTouchScope as Touch;
    let field_read = |entity: &str,
                      aspect: &worth_foundational::facade::AspectKey,
                      fields: &[worth_foundational::facade::FieldKey]| {
        query.projections().iter().any(|read| {
            read.entity() == entity
                && read.aspect_key() == aspect
                && fields.contains(read.field_key())
        }) || query
            .relations()
            .iter()
            .filter_map(|relation| relation.predicate())
            .any(|read| {
                read.field().0 == entity
                    && read.aspect_key() == aspect
                    && fields.contains(read.field_key())
            })
            || query.predicates().iter().any(|read| {
                read.field().0 == entity
                    && read.aspect_key() == aspect
                    && fields.contains(read.field_key())
            })
            || query.ordering().iter().any(|read| {
                read.field().0 == entity
                    && read.aspect_key() == aspect
                    && fields.contains(read.field_key())
            })
    };
    match touch {
        Touch::WriteField(write) => {
            write.schema() == schema
                && (query.has_extended_read_scope()
                    || field_read(
                        write.entity(),
                        write.contract().key(),
                        write.field_path().fields(),
                    ))
        }
        Touch::CreateEntity(write) | Touch::DeleteEntity(write) => {
            write.schema() == schema
                && (query.has_extended_read_scope()
                    || query.root_entity() == write.entity()
                    || query
                        .projections()
                        .iter()
                        .any(|read| read.entity() == write.entity())
                    || query
                        .relations()
                        .iter()
                        .any(|read| read.from() == write.entity() || read.to() == write.entity()))
        }
        Touch::LinkRelation(write) | Touch::UnlinkRelation(write) => {
            write.schema() == schema
                && (query.has_extended_read_scope()
                    || query.relations().iter().any(|read| {
                        read.relation() == write.relation()
                            && read.from() == write.from()
                            && read.to() == write.to()
                    }))
        }
        Touch::InitializeRelationField(write) => {
            write.relation().schema() == schema
                && (query.has_extended_read_scope()
                    || query
                        .relations()
                        .iter()
                        .any(|read| read.relation() == write.relation().relation()))
        }
        Touch::RemoveAspect(write) => {
            write.schema() == schema
                && (query.has_extended_read_scope()
                    || query.projections().iter().any(|read| {
                        read.entity() == write.entity()
                            && read.aspect_key() == write.contract().key()
                    })
                    || query.predicates().iter().any(|read| {
                        read.field().0 == write.entity()
                            && read.aspect_key() == write.contract().key()
                    })
                    || query.ordering().iter().any(|read| {
                        read.field().0 == write.entity()
                            && read.aspect_key() == write.contract().key()
                    })
                    || query
                        .relations()
                        .iter()
                        .filter_map(|read| read.predicate())
                        .any(|read| {
                            read.field().0 == write.entity()
                                && read.aspect_key() == write.contract().key()
                        }))
        }
        Touch::DeclaredDomain(_) => true,
    }
}
