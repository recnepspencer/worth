//! Borrowing views expose only the original disclosed result surface, never working nodes.
use super::*;
pub use worth_foundational::facade::AspectValue as WorthQueryMaterializedFieldValue;

#[derive(Clone, Copy)]
pub struct WorthQueryMaterializedQueryRow<'a> {
    node: &'a WorthQueryApplicationProjectionNode,
}
#[derive(Clone, Copy)]
pub struct WorthQueryMaterializedQueryField<'a> {
    field: &'a WorthQueryApplicationProjectedField,
}
#[derive(Clone, Copy)]
pub struct WorthQueryMaterializedQueryRelation<'a> {
    relation: &'a WorthQueryApplicationProjectedRelation,
}

pub(in crate::domain_computation::primary_graph::application_query::projection) fn rows(
    nodes: &[WorthQueryApplicationProjectionNode],
) -> impl ExactSizeIterator<Item = WorthQueryMaterializedQueryRow<'_>> {
    nodes
        .iter()
        .map(|node| WorthQueryMaterializedQueryRow { node })
}
impl<'a> WorthQueryMaterializedQueryRow<'a> {
    pub fn fields(&self) -> impl ExactSizeIterator<Item = WorthQueryMaterializedQueryField<'a>> {
        self.node
            .fields
            .iter()
            .map(|field| WorthQueryMaterializedQueryField { field })
    }
    pub fn relations(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryMaterializedQueryRelation<'a>> {
        self.node
            .relations
            .iter()
            .map(|relation| WorthQueryMaterializedQueryRelation { relation })
    }
}
impl<'a> WorthQueryMaterializedQueryField<'a> {
    pub fn output_name(&self) -> Option<&'a str> {
        self.field.slot_key.output_name()
    }
    pub fn result_path(&self) -> &'a str {
        &self.field.result_path
    }
    pub fn value(&self) -> &'a WorthQueryMaterializedFieldValue {
        &self.field.value
    }
}
impl<'a> WorthQueryMaterializedQueryRelation<'a> {
    pub fn output_name(&self) -> Option<&'a str> {
        self.relation.slot_key.output_name()
    }
    pub fn result_path(&self) -> &'a str {
        &self.relation.result_path
    }
    pub fn rows(&self) -> impl ExactSizeIterator<Item = WorthQueryMaterializedQueryRow<'a>> {
        self.relation
            .rows
            .iter()
            .map(|node| WorthQueryMaterializedQueryRow { node })
    }
}
