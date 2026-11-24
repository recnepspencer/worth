//! Internal access to the installed schema registries.
use super::*;

pub trait RelationalSchemaRegistryFieldsPort {
    fn into_current_publication_storage(self) -> Option<CurrentPublicationStorage>;
    fn owner_entity_kinds(&self) -> &BTreeMap<KindId, EntityKindRegistration>;
    fn owner_relation_kinds(&self) -> &BTreeMap<KindId, RelationKindRegistration>;
    fn owner_content_formats(&self) -> &crate::content::registry::RelationalContentFormatRegistry;
    fn owner_native_private_kinds(&self) -> &std::collections::BTreeSet<KindId>;
    fn owner_derived_relations(
        &self,
    ) -> &BTreeMap<
        crate::derived_relation::definition::DerivedRelationIdentity,
        std::sync::Arc<crate::derived_relation::definition::RegisteredDerivedRelation>,
    >;
    fn owner_current_publication_storage(&self) -> &Option<CurrentPublicationStorage>;
    fn owner_persistent_name_schemes(
        &self,
    ) -> &crate::lineage::data::InstalledPersistentNameSchemeRegistry;
    fn owner_kind_name_schemes(
        &self,
    ) -> &BTreeMap<(crate::lineage::data::PersistentNameSubjectKind, KindId), String>;
    fn owner_projected_member_domains(
        &self,
    ) -> &BTreeMap<
        crate::projected_member_domain::ProjectedMemberDomainIdentity,
        std::sync::Arc<crate::projected_member_domain::RegisteredProjectedMemberDomain>,
    >;
    fn owner_content_formats_mut(
        &mut self,
    ) -> &mut crate::content::registry::RelationalContentFormatRegistry;
    fn owner_entity_kinds_mut(&mut self) -> &mut BTreeMap<KindId, EntityKindRegistration>;
    fn owner_relation_kinds_mut(&mut self) -> &mut BTreeMap<KindId, RelationKindRegistration>;
}

impl RelationalSchemaRegistryFieldsPort for RelationalSchemaRegistry {
    fn into_current_publication_storage(self) -> Option<CurrentPublicationStorage> {
        self.current_publication_storage
    }
    fn owner_entity_kinds(&self) -> &BTreeMap<KindId, EntityKindRegistration> {
        &self.entity_kinds
    }
    fn owner_relation_kinds(&self) -> &BTreeMap<KindId, RelationKindRegistration> {
        &self.relation_kinds
    }
    fn owner_content_formats(&self) -> &crate::content::registry::RelationalContentFormatRegistry {
        &self.content_formats
    }
    fn owner_native_private_kinds(&self) -> &std::collections::BTreeSet<KindId> {
        &self.native_private_kinds
    }
    fn owner_derived_relations(
        &self,
    ) -> &BTreeMap<
        crate::derived_relation::definition::DerivedRelationIdentity,
        std::sync::Arc<crate::derived_relation::definition::RegisteredDerivedRelation>,
    > {
        &self.derived_relations
    }
    fn owner_current_publication_storage(&self) -> &Option<CurrentPublicationStorage> {
        &self.current_publication_storage
    }
    fn owner_persistent_name_schemes(
        &self,
    ) -> &crate::lineage::data::InstalledPersistentNameSchemeRegistry {
        &self.persistent_name_schemes
    }
    fn owner_kind_name_schemes(
        &self,
    ) -> &BTreeMap<(crate::lineage::data::PersistentNameSubjectKind, KindId), String> {
        &self.kind_name_schemes
    }
    fn owner_projected_member_domains(
        &self,
    ) -> &BTreeMap<
        crate::projected_member_domain::ProjectedMemberDomainIdentity,
        std::sync::Arc<crate::projected_member_domain::RegisteredProjectedMemberDomain>,
    > {
        &self.projected_member_domains
    }
    fn owner_content_formats_mut(
        &mut self,
    ) -> &mut crate::content::registry::RelationalContentFormatRegistry {
        &mut self.content_formats
    }
    fn owner_entity_kinds_mut(&mut self) -> &mut BTreeMap<KindId, EntityKindRegistration> {
        &mut self.entity_kinds
    }
    fn owner_relation_kinds_mut(&mut self) -> &mut BTreeMap<KindId, RelationKindRegistration> {
        &mut self.relation_kinds
    }
}
