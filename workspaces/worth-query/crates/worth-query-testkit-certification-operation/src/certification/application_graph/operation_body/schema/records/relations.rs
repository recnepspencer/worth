use super::*;

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, ExternalMapping, Principal> {
        relation(BodyModel, ExternalMapping, Self)
    }
}
pub struct DocumentMembers;
impl Relation for DocumentMembers {
    const NAME: &'static str = "DocumentMembers";
    type To = Audit;
}
impl DocumentMembers {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Document, Audit> {
        relation(BodyModel, Document, Self)
    }
}
pub struct TenantDocuments;
impl Relation for TenantDocuments {
    const NAME: &'static str = "TenantDocuments";
    type To = Document;
}
impl TenantDocuments {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Tenant, Document> {
        relation(BodyModel, Tenant, Self)
    }
}
pub struct DocumentTenant;
impl Relation for DocumentTenant {
    const NAME: &'static str = "DocumentTenant";
    type To = Tenant;
}
impl DocumentTenant {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Document, Tenant> {
        relation(BodyModel, Document, Self)
    }
}
pub struct OwnedRows;
impl Relation for OwnedRows {
    const NAME: &'static str = "OwnedRows";
    type To = Audit;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, Some(1), None, Some(1));
}
impl OwnedRows {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Document, Audit> {
        relation(BodyModel, Document, Self)
    }
}
pub struct CurrentSelection;
impl Relation for CurrentSelection {
    const NAME: &'static str = "CurrentSelection";
    type To = Snapshot;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, Some(1));
}
impl CurrentSelection {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Audit, Snapshot> {
        relation(BodyModel, Audit, Self)
    }
}
pub struct AlternateSelection;
impl Relation for AlternateSelection {
    const NAME: &'static str = "AlternateSelection";
    type To = Snapshot;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, Some(1));
}
impl AlternateSelection {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Audit, Snapshot> {
        relation(BodyModel, Audit, Self)
    }
}

impl fragment::Scalar<DraftNote> for BodyModelBindings {
    type Binding = DraftNoteBinding;
    const EQUALITY_QUERYABLE: bool = false;
}
