//! Typed external bytes and retained value, with no emission or admission authority.
use super::ApplicationExternalEffectBinding;
#[cfg(test)]
mod tests;
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
    bytes: Arc<[u8]>,
    maximum_bytes: u64,
}
impl ApplicationExternalEffectPayloadProjection {
    /// Encodes a value only. The caller still needs installed emission authority
    /// and receives no permission from successful payload validation.
    pub fn encode<Binding: ApplicationExternalEffectBinding>(
        effect: &'static str,
        value: Binding::Value,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Binding::Value: Send + Sync,
    {
        Binding::validate(&value)?;
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
            value: Arc::new(value),
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
    pub fn bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }
    pub fn maximum_bytes(&self) -> u64 {
        self.maximum_bytes
    }
}
