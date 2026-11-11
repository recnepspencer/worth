use super::*;

#[derive(Clone, Copy, Debug)]
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<PublicationAuthorizationSchema, Self> {
        entity(PublicationRecords, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (
        ResourceOwner,
        GrantGrantee,
        GrantGrantor,
        ExplicitDeny,
        ConflictingActor,
        RequestActor,
        PriorActor,
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<PublicationAuthorizationSchema, Self> {
        entity(PublicationRecords, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Resource;
impl fragment::Entity for Resource {
    const NAME: &'static str = "Resource";
    type Aspects = (ResourceFacts,);
    type Relations = ();
}
impl Resource {
    pub fn reference() -> ApplicationEntityRef<PublicationAuthorizationSchema, Self> {
        entity(PublicationRecords, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CapabilityGrant;
impl fragment::Entity for CapabilityGrant {
    const NAME: &'static str = "CapabilityGrant";
    type Aspects = (GrantFacts,);
    type Relations = (GrantResource, GrantParent);
}
impl CapabilityGrant {
    pub fn reference() -> ApplicationEntityRef<PublicationAuthorizationSchema, Self> {
        entity(GrantRecords, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ActionRecord;
impl fragment::Entity for ActionRecord {
    const NAME: &'static str = "ActionRecord";
    type Aspects = (ActionRecordFacts,);
    type Relations = (ActionResource,);
}
impl ActionRecord {
    pub fn reference() -> ApplicationEntityRef<PublicationAuthorizationSchema, Self> {
        entity(PublicationRecords, Self)
    }
}
