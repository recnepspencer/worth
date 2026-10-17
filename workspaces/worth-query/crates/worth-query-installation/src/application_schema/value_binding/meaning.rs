//! Cold lowering of value meaning from the installed catalogues. No roster is
//! stored in parallel with the declaration, executable binding or native shape.
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use worth_foundational::facade::{AspectShape, FieldRequirement};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectValueMeaning, ApplicationFieldPresence, ApplicationSchema,
    ApplicationSchemaMember, ApplicationValueMeaningDenial,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledValueMeaningDenial {
    Declaration(ApplicationValueMeaningDenial),
    NativeAspectMismatch,
    NativeFieldMismatch,
    BindingMismatch,
}

impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationSchema<Schema> {
    /// Descriptive readiness/recovery input, checked against this handle's exact
    /// installed executable bindings and native physical field closure.
    pub fn value_meaning(
        &self,
    ) -> Result<Vec<ApplicationAspectValueMeaning>, WorthQueryInstalledValueMeaningDenial> {
        use WorthQueryInstalledValueMeaningDenial as D;
        let meanings =
            ApplicationAspectValueMeaning::from_members(self.installed_declaration().members())
                .map_err(D::Declaration)?;
        if meanings.len() != self.native_contracts().len() {
            return Err(D::NativeAspectMismatch);
        }
        for meaning in &meanings {
            let native = self
                .native_contracts()
                .aspect(meaning.entity(), meaning.aspect())
                .ok_or(D::NativeAspectMismatch)?;
            let AspectShape::Struct(shape) = native.contract().shape() else {
                return Err(D::NativeAspectMismatch);
            };
            if native.contract().identity() != meaning.identity()
                || native.contract().revision() != meaning.revision()
                || shape.fields().len() != meaning.fields().len()
            {
                return Err(D::NativeAspectMismatch);
            }
            for field in meaning.fields() {
                let ApplicationSchemaMember::Field {
                    scalar_family,
                    value_type,
                    canonical,
                    unit,
                    frame,
                    ..
                } = field.member()
                else {
                    unreachable!("meaning contains only field members")
                };
                let binding = self
                    .value_bindings()
                    .field(meaning.entity(), meaning.aspect(), field.name())
                    .ok_or(D::BindingMismatch)?;
                if binding.scalar_family() != *scalar_family
                    || binding.identity().as_str() != value_type
                    || binding.canonical_contract() != canonical.as_ref()
                    || binding.unit().map(|unit| unit.as_str()) != unit.as_deref()
                    || binding.frame().map(|frame| frame.as_str()) != frame.as_deref()
                {
                    return Err(D::BindingMismatch);
                }
                let native_field = shape
                    .fields()
                    .iter()
                    .find(|native| native.key().as_str() == field.name())
                    .ok_or(D::NativeFieldMismatch)?;
                let requirement = match field.presence() {
                    ApplicationFieldPresence::Required => FieldRequirement::Required,
                    ApplicationFieldPresence::Optional => FieldRequirement::Optional,
                };
                if native_field.value_type() != *scalar_family
                    || native_field.requirement() != requirement
                    || native_field.codec() != field.codec_digest()
                {
                    return Err(D::NativeFieldMismatch);
                }
            }
        }
        Ok(meanings)
    }
}
