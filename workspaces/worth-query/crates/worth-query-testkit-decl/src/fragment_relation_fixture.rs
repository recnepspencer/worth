//! A relation record exercises arrays and nested fields under relation ownership.
use crate::fragment_fixture::{Bindings, Measurements};
use worth_query_decl::facade::application_schema::fragment::*;

pub struct Linked;
impl Fragment for Linked {
    const NAME: &'static str = "linked";
    type Bindings = Bindings;
    type Members = (Origin,);
}
pub struct Origin;
impl Entity for Origin {
    const NAME: &'static str = "origin";
    type Aspects = ();
    type Relations = (WithAspects<Connect, (Option<EdgeMeasurements>,)>,);
}
pub struct Connect;
impl Relation for Connect {
    const NAME: &'static str = "connect";
    type To = Origin;
    const SELF_EDGES: bool = false;
}
pub struct EdgeMeasurements;
impl Aspect for EdgeMeasurements {
    const NAME: &'static str = "measurements";
    type Row = Measurements;
}
