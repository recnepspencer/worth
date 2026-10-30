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
    /// installed entity-field bindings and native physical field closure.
    /// Relation aspects retain their separate grouped declaration/native meaning;
    /// this legacy entity roster does not claim whole-schema recovery readiness.
    pub fn value_meaning(
        &self,
    ) -> Result<Vec<ApplicationAspectValueMeaning>, WorthQueryInstalledValueMeaningDenial> {
        use WorthQueryInstalledValueMeaningDenial as D;
        let meanings =
            ApplicationAspectValueMeaning::from_members(self.installed_declaration().members())
                .map_err(D::Declaration)?;
        if meanings.len()
            != self
                .native_contracts()
                .contracts()
                .filter(|contract| contract.locus().owner_kind().is_entity())
                .count()
        {
            return Err(D::NativeAspectMismatch);
        }
        for meaning in &meanings {
            let native = self
                .native_contracts()
                .aspect(meaning.entity(), meaning.aspect())
                .ok_or(D::NativeAspectMismatch)?;
            if matches!(native.contract().shape(), AspectShape::Scalar(_)) != meaning.scalar_field().is_some()
                || meaning.scalar_field().is_some_and(|name| !matches!(native.binding(),
                    worth_foundational::facade::AspectBinding::EntityField { field } if field.as_str() == name))
            { return Err(D::NativeAspectMismatch); }
            if native.contract().identity() != meaning.identity()
                || native.contract().revision() != meaning.revision()
                || native.contract().absence() != meaning.absence()
                || native.fields().len() != meaning.fields().len()
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
                let key = worth_foundational::facade::FieldKey::new(field.name())
                    .ok_or(D::NativeFieldMismatch)?;
                let (native_family, native_requirement, native_codec) = native
                    .field_value_contract(&key)
                    .ok_or(D::NativeFieldMismatch)?;
                let requirement = match field.presence() {
                    ApplicationFieldPresence::Required => FieldRequirement::Required,
                    ApplicationFieldPresence::Optional => FieldRequirement::Optional,
                };
                if native_family != *scalar_family
                    || native_requirement != requirement
                    || native_codec != field.codec_digest()
                {
                    return Err(D::NativeFieldMismatch);
                }
            }
        }
        Ok(meanings)
    }
}
