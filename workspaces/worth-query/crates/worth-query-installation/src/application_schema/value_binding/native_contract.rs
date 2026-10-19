use std::any::TypeId;

use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ApplicationFrameIdentity,
    ApplicationNativeInputAdmissionDenial, ApplicationNativeInputDecodeResources,
    ApplicationSchemaBindingIdentity, ApplicationUnitIdentity, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

/// Installed scalar representation for one exact application field locus.
#[derive(Clone, Debug)]
pub struct WorthQueryInstalledApplicationValueBinding {
    schema: ApplicationSchemaBindingIdentity,
    recipe: ApplicationFieldBindingRecipe,
}

impl WorthQueryInstalledApplicationValueBinding {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        recipe: ApplicationFieldBindingRecipe,
    ) -> Self {
        Self { schema, recipe }
    }

    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn locus(&self) -> &ApplicationFieldBindingLocus {
        self.recipe.locus()
    }

    pub fn identity(&self) -> &WorthQueryPortableTypeIdentity {
        self.recipe.binding_identity()
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.recipe.scalar_family()
    }

    pub const fn unit(&self) -> Option<ApplicationUnitIdentity> {
        self.recipe.unit()
    }

    pub const fn frame(&self) -> Option<ApplicationFrameIdentity> {
        self.recipe.frame()
    }

    pub fn value_type(&self) -> TypeId {
        self.recipe.value_type()
    }

    pub fn binding_type(&self) -> TypeId {
        self.recipe.binding_type()
    }

    pub const fn is_readable(&self) -> bool {
        self.recipe.decode().is_some()
    }

    pub const fn admits_native_input(&self) -> bool {
        self.recipe.native_input_admission().is_some()
    }

    pub fn native_input_resources(
        &self,
        value: &AspectValue,
    ) -> Result<ApplicationNativeInputDecodeResources, ApplicationNativeInputAdmissionDenial> {
        let profile = self
            .recipe
            .native_input_admission()
            .ok_or(ApplicationNativeInputAdmissionDenial::InvalidValue)?;
        if value.value_family() != self.recipe.scalar_family() {
            return Err(ApplicationNativeInputAdmissionDenial::InvalidValue);
        }
        let bytes = u64::try_from(value.owned_allocation_capacity_bytes())
            .map_err(|_| ApplicationNativeInputAdmissionDenial::CapacityOverflow)?;
        profile
            .requirements(bytes)
            .ok_or(ApplicationNativeInputAdmissionDenial::CapacityOverflow)
    }

    pub const fn is_identity(&self) -> bool {
        self.recipe.identity_capable()
    }

    pub const fn is_signed_aggregate(&self) -> bool {
        self.recipe.signed_aggregate_decode().is_some()
    }

    pub fn encode<Value: 'static>(
        &self,
        value: &Value,
    ) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        (self.recipe.encode())(value)
    }

    pub fn decode<Value: 'static>(
        &self,
        value: &AspectValue,
    ) -> Result<Value, ApplicationValueDecodeDenial> {
        let decode = self
            .recipe
            .decode()
            .ok_or(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: self.recipe.binding_identity().clone(),
            })?;
        decode(value)?
            .downcast::<Value>()
            .map(|value| *value)
            .map_err(|_| ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: self.recipe.binding_identity().clone(),
            })
    }

    /// Readmits a retained native literal through this exact installed decoder
    /// and validator. This cold binding check does not grant mutation authority.
    pub fn validate_encoded(
        &self,
        value: &AspectValue,
    ) -> Result<(), ApplicationValueDecodeDenial> {
        let decode = self
            .recipe
            .decode()
            .ok_or(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: self.recipe.binding_identity().clone(),
            })?;
        decode(value).map(|_| ())
    }
}
