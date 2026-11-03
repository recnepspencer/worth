//! One concrete Native adapter for all installed typed content bindings.
use std::any::Any;
use worth_foundational::facade::ContentFormat;
use worth_query_declaration::facade::application_schema::{
    ApplicationContentBindingRecipe as Recipe, ApplicationContentCodecDenial as CallbackDenial,
    ApplicationContentControl as ControlPort, ApplicationContentEncoder as EncoderPort,
};
use worth_relational::facade::content::{
    RelationalContentCodec, RelationalContentControl as NativeControl,
    RelationalContentDenial as Denial, RelationalContentSink as NativeSink,
};

#[derive(Debug)]
pub(super) struct Codec {
    pub(super) descriptor: ContentFormat,
    pub(super) recipe: Recipe,
    pub(super) maximum: usize,
}
impl RelationalContentCodec for Codec {
    fn format(&self) -> &ContentFormat {
        &self.descriptor
    }
    fn maximum_frame_bytes(&self) -> usize {
        self.maximum
    }
    fn payload_length(
        &self,
        value: &dyn Any,
        native: &mut NativeControl<'_, '_, '_>,
    ) -> Result<usize, Denial> {
        let mut control = Control::new(native);
        let result = (self.recipe.length())(value, &mut control);
        control.finish(result)
    }
    fn encode(
        &self,
        value: &dyn Any,
        sink: &mut NativeSink<'_>,
        native: &mut NativeControl<'_, '_, '_>,
    ) -> Result<(), Denial> {
        let mut control = Control::new(native);
        let result = (self.recipe.encode())(
            value,
            &mut Encoder {
                sink,
                control: &mut control,
            },
        );
        control.finish(result)
    }
    fn validate(&self, bytes: &[u8], native: &mut NativeControl<'_, '_, '_>) -> Result<(), Denial> {
        let mut control = Control::new(native);
        let result = (self.recipe.validate())(bytes, &mut control);
        control.finish(result)
    }
    fn verify_encoding(
        &self,
        value: &dyn Any,
        bytes: &[u8],
        native: &mut NativeControl<'_, '_, '_>,
    ) -> Result<(), Denial> {
        let mut control = Control::new(native);
        let result = (self.recipe.verify())(value, bytes, &mut control);
        control.finish(result)
    }
}
struct Control<'call, 'borrow, 'scope, 'lease> {
    native: &'call mut NativeControl<'borrow, 'scope, 'lease>,
    first_denial: Option<Denial>,
}
impl<'call, 'borrow, 'scope, 'lease> Control<'call, 'borrow, 'scope, 'lease> {
    fn new(native: &'call mut NativeControl<'borrow, 'scope, 'lease>) -> Self {
        Self {
            native,
            first_denial: None,
        }
    }
    fn stop(&mut self, cause: Denial) -> CallbackDenial {
        if self.first_denial.is_none() {
            self.first_denial = Some(cause);
        }
        CallbackDenial::AdmissionStopped
    }
    fn finish<T>(&mut self, result: Result<T, CallbackDenial>) -> Result<T, Denial> {
        if let Some(cause) = self.first_denial.take() {
            return Err(cause);
        }
        self.native.checkpoint(0)?;
        result.map_err(|cause| match cause {
            CallbackDenial::AdmissionStopped => Denial::EncodingInconsistency,
            CallbackDenial::InputTypeMismatch => Denial::InputTypeMismatch,
            CallbackDenial::NonCanonicalContent => Denial::NonCanonicalContent,
            CallbackDenial::EncodingInconsistency => Denial::EncodingInconsistency,
            CallbackDenial::ArithmeticOverflow => Denial::ArithmeticOverflow,
            CallbackDenial::SizeLimit { observed, maximum } => {
                Denial::SizeLimit { observed, maximum }
            }
        })
    }
}
impl ControlPort for Control<'_, '_, '_, '_> {
    fn checkpoint(&mut self, units: u64) -> Result<(), CallbackDenial> {
        if self.first_denial.is_some() {
            return Err(CallbackDenial::AdmissionStopped);
        }
        self.native
            .checkpoint(units)
            .map_err(|cause| self.stop(cause))
    }
}
struct Encoder<'sink, 'buffer, 'control, 'call, 'borrow, 'scope, 'lease> {
    sink: &'sink mut NativeSink<'buffer>,
    control: &'control mut Control<'call, 'borrow, 'scope, 'lease>,
}
impl ControlPort for Encoder<'_, '_, '_, '_, '_, '_, '_> {
    fn checkpoint(&mut self, units: u64) -> Result<(), CallbackDenial> {
        self.control.checkpoint(units)
    }
}
impl EncoderPort for Encoder<'_, '_, '_, '_, '_, '_, '_> {
    fn append(&mut self, bytes: &[u8]) -> Result<(), CallbackDenial> {
        if self.control.first_denial.is_some() {
            return Err(CallbackDenial::AdmissionStopped);
        }
        self.sink
            .append(bytes, self.control.native)
            .map_err(|cause| self.control.stop(cause))
    }
}
