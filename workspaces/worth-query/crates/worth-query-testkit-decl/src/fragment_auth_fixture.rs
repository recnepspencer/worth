//! Native fragment courts use these declared principal and scope relationships.
use super::fragment_fixture::*;
pub use bindings::{MappingData, PrincipalData};
pub use grants::{ChangeForm, FieldsOnly, IdentityBinding, IdentityValueBinding, InspectOptional};
use worth_query_decl::facade::authentication::*;
mod bindings;
mod grants;

pub struct Authentication;
impl Fragment for Authentication {
    const NAME: &'static str = "authentication";
    type Bindings = Bindings;
    type Members = (
        Mapping,
        Principal,
        OptionalItem,
        IdentityBinding,
        grants::Access,
        ChangeForm,
        InspectOptional,
        FieldsOnly,
    );
}
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "mapping";
    type Aspects = (MappingIdentity,);
    type Relations = (MapsTo,);
}
pub struct MappingIdentity;
impl Aspect for MappingIdentity {
    const NAME: &'static str = "mapping_identity";
    type Row = MappingData;
}
pub struct MapsTo;
impl Relation for MapsTo {
    const NAME: &'static str = "maps_to";
    type To = Principal;
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "principal";
    type Aspects = (Identity,);
    type Relations = (OwnsItem, OwnsOptional);
}
pub struct Identity;
impl Aspect for Identity {
    const NAME: &'static str = "principal_identity";
    type Row = PrincipalData;
}
pub struct OptionalItem;
impl Entity for OptionalItem {
    const NAME: &'static str = "optional_item";
    type Aspects = (OptionalIdentity, Option<Exclusive<OptionalForm>>);
    type Relations = ();
}
pub struct ItemIdentity;
impl Aspect for ItemIdentity {
    const NAME: &'static str = "item_identity";
    type Row = PrincipalData;
}
pub struct OptionalIdentity;
impl Aspect for OptionalIdentity {
    const NAME: &'static str = "optional_identity";
    type Row = PrincipalData;
}
pub struct OptionalForm;
impl OneOf for OptionalForm {
    type Row = FormValue;
    type Cases = (OptionalMeasured, OptionalNamed);
}
pub struct OptionalMeasured;
impl Aspect for OptionalMeasured {
    const NAME: &'static str = "optional_measured";
    type Row = Measurements;
}
pub struct OptionalNamed;
impl Aspect for OptionalNamed {
    const NAME: &'static str = "optional_named";
    type Row = Caption;
}

pub struct OwnsItem;
impl Relation for OwnsItem {
    const NAME: &'static str = "owns_item";
    type To = Item;
}
pub struct OwnsOptional;
impl Relation for OwnsOptional {
    const NAME: &'static str = "owns_optional";
    type To = OptionalItem;
}

pub fn samples(value: std::any::TypeId) -> Option<Vec<Box<dyn BindingSample>>> {
    use std::any::TypeId;
    let external =
        || WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "person").unwrap();
    if value == TypeId::of::<PrincipalData>() {
        return Some(vec![super::fragment_fixture::row_sample(PrincipalData {
            id: 13,
        })]);
    }
    if value == TypeId::of::<MappingData>() {
        return Some(vec![super::fragment_fixture::row_sample(MappingData {
            external: external(),
            status: WorthQueryPrincipalMappingStatus::Enabled,
        })]);
    }
    if value == TypeId::of::<WorthQueryExternalPrincipalIdentity>() {
        return Some(vec![super::fragment_fixture::scalar_sample(external())]);
    }
    if value == TypeId::of::<WorthQueryPrincipalMappingStatus>() {
        return Some(vec![super::fragment_fixture::scalar_sample(
            WorthQueryPrincipalMappingStatus::Enabled,
        )]);
    }
    None
}
