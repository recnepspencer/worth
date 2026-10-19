//! Typed external bytes and retained value, with no emission or admission authority.
use super::ApplicationExternalEffectBinding;
use crate::application_schema::ApplicationValueEncodeDenial;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use std::{
    any::{Any, TypeId},
    sync::Arc,
};

pub struct ApplicationExternalEffectPayloadProjection {
    effect: &'static str,
    identity: WorthQueryPortableTypeIdentity,
    value_type: TypeId,
    value: Arc<dyn Any + Send + Sync>,
    retained_bytes: u64,
    measure: fn(&(dyn Any + Send + Sync)) -> Option<u64>,
    bytes: Arc<[u8]>,
    maximum_bytes: u64,
}
impl ApplicationExternalEffectPayloadProjection {
    /// Encodes a value only. The caller still needs installed emission authority
    /// and must validate and fund the payload before constructing it.
    pub fn encode<Binding: ApplicationExternalEffectBinding>(
        effect: &'static str,
        value: Binding::Value,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Binding::Value: Send + Sync,
    {
        let bytes = Binding::external_effect_bytes(&value);
        if Binding::MAX_EXTERNAL_BYTES == 0
            || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > Binding::MAX_EXTERNAL_BYTES
        {
            return Err(ApplicationValueEncodeDenial::CodecRejected {
                binding_identity: Binding::IDENTITY,
                reason: "external payload exceeds its declared byte bound",
            });
        }
        Ok(Self {
            effect,
            identity: Binding::IDENTITY,
            value_type: TypeId::of::<Binding::Value>(),
            retained_bytes: Binding::retained_bytes(&value),
            value: Arc::new(value),
            measure: measure::<Binding>,
            bytes: bytes.into(),
            maximum_bytes: Binding::MAX_EXTERNAL_BYTES,
        })
    }
    pub fn effect(&self) -> &'static str {
        self.effect
    }
    pub fn identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.identity
    }
    pub fn value_type(&self) -> TypeId {
        self.value_type
    }
    pub fn value(&self) -> Arc<dyn Any + Send + Sync> {
        Arc::clone(&self.value)
    }
    pub fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
    pub fn measure(&self) -> fn(&(dyn Any + Send + Sync)) -> Option<u64> {
        self.measure
    }
    pub fn bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }
    pub fn maximum_bytes(&self) -> u64 {
        self.maximum_bytes
    }
}
fn measure<Binding: ApplicationExternalEffectBinding>(
    value: &(dyn Any + Send + Sync),
) -> Option<u64> {
    value
        .downcast_ref::<Binding::Value>()
        .map(Binding::retained_bytes)
}
