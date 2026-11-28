pub mod foundational_contract_basis;
pub mod schema_plan_terms;

use crate::identity::data::KindId;
use crate::schema::data::aspect_merge_policy::AspectMergePolicyDeclaration;
use crate::schema::data::identity_declarations::IdentityBasisDeclaration;
use crate::schema::data::{
    AspectContractPlanRevision, DeclaredAspectContractBinding, SchemaRegistryError,
};

use foundational_contract_basis::mix_foundational_contract_basis;
use schema_plan_terms::{
    mix_aspect_binding_terms, mix_identity_declaration_terms, mix_merge_policy_declaration_terms,
    RevisionHasher,
};

pub(super) fn derive_plan_revision(
    kind_id: KindId,
    aspects: &[DeclaredAspectContractBinding],
    identity_declarations: &[IdentityBasisDeclaration],
    merge_policy_declarations: &[AspectMergePolicyDeclaration],
) -> Result<AspectContractPlanRevision, SchemaRegistryError> {
    let mut revision = RevisionHasher::new();
    for aspect in aspects {
        mix_aspect_binding_terms(&mut revision, aspect);
        mix_foundational_contract_basis(&mut revision, kind_id, aspect)?;
    }
    for declaration in identity_declarations {
        mix_identity_declaration_terms(&mut revision, declaration);
    }
    for declaration in merge_policy_declarations {
        mix_merge_policy_declaration_terms(&mut revision, declaration);
    }
    Ok(AspectContractPlanRevision(revision.finish()))
}

#[cfg(test)]
mod tests {
    use worth_foundational::{
        aspects, AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
        ScalarAspectType,
    };

    use super::derive_plan_revision;
    use crate::identity::data::KindId;
    use crate::schema::data::{AspectBinding, DeclaredAspectContractBinding};

    #[test]
    fn revision_changes_when_foundational_scalar_contract_family_changes() {
        let string_revision = revision_for(contract("profile.score", ScalarAspectType::String));
        let int_revision = revision_for(contract("profile.score", ScalarAspectType::Int64));

        assert_ne!(string_revision, int_revision);
    }

    #[test]
    fn revision_changes_when_foundational_struct_field_law_changes() {
        let required_revision = revision_for(struct_contract(
            "profile.summary",
            aspects()
                .struct_fields()
                .required("summary", ScalarAspectType::String)
                .finish()
                .expect("valid required struct shape"),
        ));
        let optional_revision = revision_for(struct_contract(
            "profile.summary",
            aspects()
                .struct_fields()
                .optional("summary", ScalarAspectType::String)
                .finish()
                .expect("valid optional struct shape"),
        ));

        assert_ne!(required_revision, optional_revision);
    }

    fn revision_for(contract: AspectContract) -> crate::schema::data::AspectContractPlanRevision {
        derive_plan_revision(
            KindId(7),
            &[DeclaredAspectContractBinding {
                binding: AspectBinding::EntityField {
                    field: worth_foundational::facade::FieldKey::new("summary")
                        .expect("valid test key"),
                },
                contract,
            }],
            &[],
            &[],
        )
        .expect("contract canonical basis should derive a schema aspect plan revision")
    }

    fn contract(key: &str, scalar: ScalarAspectType) -> AspectContract {
        AspectContract::scalar(
            AspectKey::new(key).expect("valid aspect key"),
            AspectIdentity(1),
            AspectContractRevision(1),
            scalar,
        )
    }

    fn struct_contract(key: &str, shape: worth_foundational::StructAspectShape) -> AspectContract {
        AspectContract::struct_aspect(
            AspectKey::new(key).expect("valid aspect key"),
            AspectIdentity(1),
            AspectContractRevision(1),
            shape,
        )
    }
}
