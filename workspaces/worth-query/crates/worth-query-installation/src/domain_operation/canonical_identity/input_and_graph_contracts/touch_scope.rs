use super::*;

pub(super) fn hash_touch_scope(hasher: &mut impl Sink, scope: &WorthQueryOperationTouchScope) {
    match scope {
        WorthQueryOperationTouchScope::InitializeRelationField(field) => {
            hash_text_field(hasher, "touch-scope", "initialize-relation-field");
            let relation = field.relation();
            hash_binding(hasher, relation.schema());
            hash_text_field(hasher, "touch-relation", relation.relation());
            hash_text_field(hasher, "touch-from", relation.from());
            hash_text_field(hasher, "touch-to", relation.to());
            hash_text_field(
                hasher,
                "touch-contract-canonical-material",
                field.canonical_contract_material(),
            );
            for part in field.field_path().fields() {
                hash_text_field(hasher, "touch-field-path-part", part.as_str());
            }
        }
        WorthQueryOperationTouchScope::CreateEntity(entity)
        | WorthQueryOperationTouchScope::DeleteEntity(entity) => {
            let kind = if matches!(scope, WorthQueryOperationTouchScope::CreateEntity(_)) {
                "create-entity"
            } else {
                "delete-entity"
            };
            hash_text_field(hasher, "touch-scope", kind);
            hash_binding(hasher, entity.schema());
            hash_text_field(hasher, "touch-entity", entity.entity());
        }
        WorthQueryOperationTouchScope::WriteField(field) => {
            hash_text_field(hasher, "touch-scope", "write-field");
            hash_binding(hasher, field.schema());
            hash_text_field(hasher, "touch-entity", field.entity());
            hash_text_field(
                hasher,
                "touch-contract-canonical-material",
                field.canonical_contract_material(),
            );
            for part in field.field_path().fields() {
                hash_text_field(hasher, "touch-field-path-part", part.as_str());
            }
        }
        WorthQueryOperationTouchScope::LinkRelation(relation)
        | WorthQueryOperationTouchScope::UnlinkRelation(relation) => {
            let kind = if matches!(scope, WorthQueryOperationTouchScope::LinkRelation(_)) {
                "link-relation"
            } else {
                "unlink-relation"
            };
            hash_text_field(hasher, "touch-scope", kind);
            hash_binding(hasher, relation.schema());
            hash_text_field(hasher, "touch-relation", relation.relation());
            hash_text_field(hasher, "touch-from", relation.from());
            hash_text_field(hasher, "touch-to", relation.to());
        }
        WorthQueryOperationTouchScope::DeclaredDomain(identity) => {
            hash_text_field(hasher, "touch-scope", "declared-domain");
            hash_text_field(hasher, "touch-domain-identity", identity.as_str());
        }
    }
}
