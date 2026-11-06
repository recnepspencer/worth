//! The native ownership fixture owns its model roster and reuses checked topology rows.
use super::planar_topology::*;
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
use worth_query_decl::facade::authentication::*;
pub struct TopologyRecords;
impl fragment::Fragment for TopologyRecords {
    const NAME: &'static str = "topology_records";
    type Bindings = TopologyRecordsBindings;
    type Members = (ExternalPrincipalMapping, Principal, Body);
}

#[derive(Clone, Copy)]
pub struct ExternalPrincipalMapping;
impl fragment::Entity for ExternalPrincipalMapping {
    const NAME: &'static str = "ExternalPrincipalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalPrincipalMapping {
    pub fn reference<Schema: TopologyModelBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(TopologyRecords, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference<Schema: TopologyModelBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(TopologyRecords, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Body;
impl fragment::Entity for Body {
    const NAME: &'static str = "Body";
    type Aspects = (
        PlanarPosition,
        Geometry,
        Option<super::optional_aspects::SourceNotes>,
    );
    type Relations = (
        PlanarSuccessor,
        PlanarDiscoverySource,
        UndeclaredPlanarRelation,
        PlanarCoupling,
        super::facts::BodyEntrySet,
    );
}
impl Body {
    pub fn reference<Schema: TopologyModelBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(TopologyRecords, Self)
    }
}
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference<Schema: TopologyModelBinding>(
    ) -> AspectSelection<TopologyRecords, ExternalPrincipalMapping, Self> {
        aspect(TopologyRecords, ExternalPrincipalMapping, Self)
    }
}

pub struct PrincipalFacts;
impl fragment::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference<Schema: TopologyModelBinding>(
    ) -> AspectSelection<TopologyRecords, Principal, Self> {
        aspect(TopologyRecords, Principal, Self)
    }
}

pub struct PlanarPosition;
impl fragment::Aspect for PlanarPosition {
    const NAME: &'static str = "PlanarPosition";
    type Row = PlanarPositionValue;
}
impl PlanarPosition {
    pub fn reference<Schema: TopologyModelBinding>() -> AspectSelection<TopologyRecords, Body, Self>
    {
        aspect(TopologyRecords, Body, Self)
    }
}

pub struct Geometry;
impl fragment::Aspect for Geometry {
    const NAME: &'static str = "Geometry";
    type Row = GeometryValue;
}
impl Geometry {
    pub fn reference<Schema: TopologyModelBinding>() -> AspectSelection<TopologyRecords, Body, Self>
    {
        aspect(TopologyRecords, Body, Self)
    }
}

pub type ExternalIdentityField = FieldSelection<
    TopologyRecords,
    ExternalPrincipalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    TopologyRecords,
    ExternalPrincipalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentity = FieldSelection<TopologyRecords, Principal, PrincipalFacts, u64>;
pub type BodyKey = FieldSelection<TopologyRecords, Body, PlanarPosition, String>;
pub type PositionX = FieldSelection<TopologyRecords, Body, PlanarPosition, PositiveLength>;
pub type PositionY = FieldSelection<TopologyRecords, Body, PlanarPosition, PositiveLength>;
pub type Length = FieldSelection<TopologyRecords, Body, Geometry, PositiveLength>;
