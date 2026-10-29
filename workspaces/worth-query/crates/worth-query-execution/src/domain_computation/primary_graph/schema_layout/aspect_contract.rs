//! Schema-owned nested contract lookup, with admission before each tree read.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn aspect_binding(
        &self,
        entity: &str,
        aspect: &AspectKey,
    ) -> Option<&DeclaredAspectContractBinding> {
        match self.aspect_binding_with_admission(entity, aspect, &mut |_, _| {
            Ok::<_, std::convert::Infallible>(())
        }) {
            Ok(binding) => binding,
            Err(never) => match never {},
        }
    }
    pub(in crate::domain_computation) fn aspect_contract(
        &self,
        entity: &str,
        aspect: &AspectKey,
    ) -> Option<&AspectContract> {
        match self.aspect_contract_with_admission(entity, aspect, &mut |_, _| {
            Ok::<_, std::convert::Infallible>(())
        }) {
            Ok(contract) => contract,
            Err(never) => match never {},
        }
    }

    pub(in crate::domain_computation) fn aspect_contract_with_admission<E>(
        &self,
        entity: &str,
        aspect: &AspectKey,
        admit: &mut impl FnMut(usize, &str) -> Result<(), E>,
    ) -> Result<Option<&AspectContract>, E> {
        Ok(self
            .aspect_binding_with_admission(entity, aspect, admit)?
            .map(|binding| &binding.contract))
    }

    pub(in crate::domain_computation) fn aspect_binding_with_admission<E>(
        &self,
        entity: &str,
        aspect: &AspectKey,
        admit: &mut impl FnMut(usize, &str) -> Result<(), E>,
    ) -> Result<Option<&DeclaredAspectContractBinding>, E> {
        admit(self.aspect_contracts.len(), entity)?;
        let Some(contracts) = self.aspect_contracts.get(entity) else {
            return Ok(None);
        };
        admit(contracts.len(), aspect.as_str())?;
        Ok(contracts.get(aspect))
    }
}
