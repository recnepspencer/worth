//! A present optional column carries its SDK-owned parent occurrence.
use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationEffectProgramBuilder as Builder,
    WorthQueryApplicationOptionalFieldWrite as Write,
    WorthQueryApplicationRealizedEffect as Effect,
};
use worth_foundational::facade::{AspectValue, PortableAspectContractBasis};
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus as Locus;
impl<S, O, I, Scope> Builder<S, O, I, Scope> {
    pub(super) fn retain_record_presence(
        &mut self,
        field: &Locus,
        position: usize,
    ) -> Result<(), Denial> {
        let Some(locator) = self.layout.record_presence_locator(field).cloned() else {
            return Ok(());
        };
        let contract = self
            .layout
            .aspect_contract(field.entity(), locator.aspect().aspect_key())
            .map(PortableAspectContractBasis::from_contract)
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, field.field()))?;
        let Some(Effect::PatchOptionalEntityFields { fields, .. }) = self.effects.get_mut(position)
        else {
            return Err(Denial::new(Kind::ConflictingEffectStep, field.field()));
        };
        fields.insert(
            locator,
            Write {
                contract,
                value: Some(AspectValue::Bool(true)),
            },
        );
        Ok(())
    }
}
