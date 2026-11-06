use super::*;

pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, ExternalMapping, Principal>
    {
        relation(PublicationRecords, ExternalMapping, Self)
    }
}
pub struct ResourceOwner;
impl fragment::Relation for ResourceOwner {
    const NAME: &'static str = "ResourceOwner";
    type To = Resource;
}
impl ResourceOwner {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, Resource> {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct GrantGrantee;
impl fragment::Relation for GrantGrantee {
    const NAME: &'static str = "GrantGrantee";
    type To = CapabilityGrant;
}
impl GrantGrantee {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, CapabilityGrant>
    {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct GrantGrantor;
impl fragment::Relation for GrantGrantor {
    const NAME: &'static str = "GrantGrantor";
    type To = CapabilityGrant;
}
impl GrantGrantor {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, CapabilityGrant>
    {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct GrantResource;
impl fragment::Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Resource;
}
impl GrantResource {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, CapabilityGrant, Resource>
    {
        relation(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct GrantParent;
impl fragment::Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = CapabilityGrant;
}
impl GrantParent {
    pub fn reference() -> ApplicationRelationRef<
        PublicationAuthorizationSchema,
        Self,
        CapabilityGrant,
        CapabilityGrant,
    > {
        relation(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct ExplicitDeny;
impl fragment::Relation for ExplicitDeny {
    const NAME: &'static str = "ExplicitDeny";
    type To = Resource;
}
impl ExplicitDeny {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, Resource> {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct ConflictingActor;
impl fragment::Relation for ConflictingActor {
    const NAME: &'static str = "ConflictingActor";
    type To = Resource;
}
impl ConflictingActor {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, Resource> {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct RequestActor;
impl fragment::Relation for RequestActor {
    const NAME: &'static str = "RequestActor";
    type To = ActionRecord;
}
impl RequestActor {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, ActionRecord> {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct PriorActor;
impl fragment::Relation for PriorActor {
    const NAME: &'static str = "PriorActor";
    type To = ActionRecord;
}
impl PriorActor {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, Principal, ActionRecord> {
        relation(PublicationRecords, Principal, Self)
    }
}
pub struct ActionResource;
impl fragment::Relation for ActionResource {
    const NAME: &'static str = "ActionResource";
    type To = Resource;
}
impl ActionResource {
    pub fn reference(
    ) -> ApplicationRelationRef<PublicationAuthorizationSchema, Self, ActionRecord, Resource> {
        relation(PublicationRecords, ActionRecord, Self)
    }
}

pub struct GrantRecords;
impl Fragment for GrantRecords {
    const NAME: &'static str = "publication_grants";
    type Bindings = GrantBindings;
    type Members = (CapabilityGrant,);
}
pub struct GrantBindings;
impl BindingSet for GrantBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for GrantBindings {
    type Binding = StringApplicationValueBinding;
}
impl Scalar<u64> for GrantBindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<GrantKey> for GrantBindings {
    type Binding = GrantKeyBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantKey(String);
impl GrantKey {
    fn text(&self) -> worth_foundational::facade::InternedString {
        self.0.clone().into()
    }
    fn checked(value: worth_foundational::facade::InternedString) -> Option<Self> {
        match value {
            worth_foundational::facade::InternedString::Stored(value) => {
                let value = value.as_string();
                Some(Self(value))
            }
            worth_foundational::facade::InternedString::Raw(value) => Some(Self(value)),
            worth_foundational::facade::InternedString::Symbol(_) => None,
        }
    }
}
impl From<String> for GrantKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}
worth_query_declaration::worth_query_value_binding!(pub GrantKeyBinding for GrantKey { identity: "worth.query.fixture.publication.grant-key.v1", scalar: String, encode: GrantKey::text, decode: GrantKey::checked, });
