use super::*;

#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(BodyModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(BodyModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Document;
impl Entity for Document {
    const NAME: &'static str = "Document";
    type Aspects = (DocumentFacts, Option<DocumentDraftFacts>);
    type Relations = (DocumentMembers, DocumentTenant, OwnedRows);
}
impl Document {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(BodyModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Audit;
impl Entity for Audit {
    const NAME: &'static str = "Audit";
    type Aspects = (AuditFacts, Option<AuditActorFacts>);
    type Relations = (CurrentSelection, AlternateSelection);
}
impl Audit {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(BodyModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Snapshot;
impl Entity for Snapshot {
    const NAME: &'static str = "Snapshot";
    type Aspects = (SnapshotFacts,);
    type Relations = ();
}
impl Snapshot {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(BodyModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Tenant;
impl Entity for Tenant {
    const NAME: &'static str = "Tenant";
    type Aspects = (TenantFacts,);
    type Relations = (TenantDocuments,);
}
impl Tenant {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationEntityRef<Schema, Self> {
        entity(BodyModel, Self)
    }
}
