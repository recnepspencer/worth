use super::*;

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<NumericSchema, Self, ExternalMapping, Principal> {
        relation(NumericModel, ExternalMapping, Self)
    }
}
pub struct AuthoredReference;
impl Relation for AuthoredReference {
    const NAME: &'static str = "AuthoredReference";
    type To = Number;
}
impl AuthoredReference {
    pub fn reference() -> ApplicationRelationRef<NumericSchema, Self, Number, Number> {
        relation(NumericModel, Number, Self)
    }
}
pub struct GrantResource;
impl Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Number;
}
impl GrantResource {
    pub fn reference() -> ApplicationRelationRef<NumericSchema, Self, Grant, Number> {
        relation(NumericModel, Grant, Self)
    }
}
pub struct GrantGrantee;
impl Relation for GrantGrantee {
    const NAME: &'static str = "GrantGrantee";
    type To = Grant;
}
impl GrantGrantee {
    pub fn reference() -> ApplicationRelationRef<NumericSchema, Self, Principal, Grant> {
        relation(NumericModel, Principal, Self)
    }
}
pub struct GrantGrantor;
impl Relation for GrantGrantor {
    const NAME: &'static str = "GrantGrantor";
    type To = Grant;
}
impl GrantGrantor {
    pub fn reference() -> ApplicationRelationRef<NumericSchema, Self, Principal, Grant> {
        relation(NumericModel, Principal, Self)
    }
}
pub struct GrantParent;
impl Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = Grant;
}
impl GrantParent {
    pub fn reference() -> ApplicationRelationRef<NumericSchema, Self, Grant, Grant> {
        relation(NumericModel, Grant, Self)
    }
}
