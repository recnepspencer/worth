//! Stack codec slots dispatch once per scalar binding, never per schema or member.
use super::ValueDecodeSession;
use std::marker::PhantomData;
use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationValueEncodeDenial,
};
pub(in crate::domain_computation::primary_graph) trait FieldDecoder {
    fn discard(&mut self);
    fn decode(
        &mut self,
        session: &ValueDecodeSession<'_>,
        entity: &str,
        aspect: &str,
        field: &str,
        raw: &AspectValue,
    );
    fn decode_relation(
        &mut self,
        session: &ValueDecodeSession<'_>,
        entity: &str,
        aspect: &str,
        field: &str,
        raw: &AspectValue,
    );
}
pub(in crate::domain_computation::primary_graph) struct ScalarFieldDecoder<
    B: ApplicationReadableScalarValueBinding,
> {
    pub(in crate::domain_computation::primary_graph) value: Option<B::Value>,
    marker: PhantomData<fn() -> B>,
}
impl<B: ApplicationReadableScalarValueBinding> ScalarFieldDecoder<B> {
    #[inline(always)]
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        Self {
            value: None,
            marker: PhantomData,
        }
    }
}
impl<B: ApplicationReadableScalarValueBinding> FieldDecoder for ScalarFieldDecoder<B> {
    fn discard(&mut self) {
        self.value = None;
    }
    fn decode(
        &mut self,
        session: &ValueDecodeSession<'_>,
        entity: &str,
        aspect: &str,
        field: &str,
        raw: &AspectValue,
    ) {
        self.value = session.decode::<B>(entity, aspect, field, raw);
    }
    fn decode_relation(
        &mut self,
        session: &ValueDecodeSession<'_>,
        entity: &str,
        aspect: &str,
        field: &str,
        raw: &AspectValue,
    ) {
        self.value = session.decode_relation::<B>(entity, aspect, field, raw);
    }
}
pub(in crate::domain_computation::primary_graph) trait FieldEncoder {
    fn validate(
        &self,
    ) -> Result<(), worth_query_installation::facade::ApplicationValueValidationDenial>;
    fn encode_value(&self) -> Result<AspectValue, ApplicationValueEncodeDenial>;
    fn encode(&self) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        self.validate()?;
        self.encode_value()
    }
}
pub(in crate::domain_computation::primary_graph) struct ScalarFieldEncoder<
    'a,
    B: ApplicationScalarValueBinding,
> {
    value: &'a B::Value,
    marker: PhantomData<fn() -> B>,
}
impl<'a, B: ApplicationScalarValueBinding> ScalarFieldEncoder<'a, B> {
    #[inline(always)]
    pub(in crate::domain_computation::primary_graph) fn new(value: &'a B::Value) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }
}
impl<B: ApplicationScalarValueBinding> FieldEncoder for ScalarFieldEncoder<'_, B> {
    fn validate(
        &self,
    ) -> Result<(), worth_query_installation::facade::ApplicationValueValidationDenial> {
        B::validate(self.value)
    }
    fn encode_value(&self) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        B::encode(self.value)
    }
}
