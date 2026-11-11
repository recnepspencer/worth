//! Portable contract and declaration storage; shared identity leaves are inputs.
use super::*;
use worth_foundational::facade::{AspectBinding, AspectShape};

pub(super) fn quote(
    memory: &mut Quote<'_, '_, '_>,
    declarations: &KindAspectContractDeclarations,
) -> Result<(), RelationalRecoveryStop> {
    memory.array::<DurableDeclaredAspectBinding>(declarations.aspects.len())?;
    for declaration in &declarations.aspects {
        if let AspectBinding::EntityField { field } | AspectBinding::RelationField { field } =
            &declaration.binding
        {
            memory.text(field.as_str())?;
        }
        contract(memory, &declaration.contract)?;
    }
    memory.array::<IdentityBasisDeclaration>(declarations.identity_declarations.len())?;
    for declaration in &declarations.identity_declarations {
        memory.poll()?;
        if let crate::merge::data::IdentityBasisScope::AspectKey(key) = &declaration.scope {
            memory.text(key.as_str())?;
        }
        // DeclaredIdentityKeys and custom names clone their original shared
        // DATA owners, not new element/text/control allocations.
    }
    memory.array::<AspectMergePolicyDeclaration>(declarations.merge_policy_declarations.len())?;
    for declaration in &declarations.merge_policy_declarations {
        memory.text(declaration.aspect_key.as_str())?;
    }
    Ok(())
}

pub(super) fn contract(
    memory: &mut Quote<'_, '_, '_>,
    contract: &AspectContract,
) -> Result<(), RelationalRecoveryStop> {
    memory.text(contract.key().as_str())?;
    if let AspectShape::Struct(shape) = contract.shape() {
        memory.add(
            PortableAspectContract::decode_field_roster_storage_bytes(shape.fields().len())
                .ok_or_else(super::overflow)?,
        )?;
        for field in shape.fields() {
            memory.text(field.key().as_str())?;
        }
    }
    Ok(())
}
