use crate::{ChargedBytes, ExecutionRetainedResult};

/// A checked map's current input or a genuine prior native result. This
/// declaration supplies no input-equality or application currentness proof.
pub enum RetainedMapInput<T, R> {
    Evaluate(T),
    Carry(ExecutionRetainedResult<R>),
}

impl<T: ChargedBytes, R> ChargedBytes for RetainedMapInput<T, R> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Evaluate(value) => value.additional_charged_bytes(),
            Self::Carry(value) => value.additional_charged_bytes(),
        }
    }
}
