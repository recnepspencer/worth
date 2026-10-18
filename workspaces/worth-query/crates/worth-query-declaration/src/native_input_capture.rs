//! Closed platform integration for Execution-owned immutable input backing.
//! Not part of either declaration audience facade. The exact native caller is
//! sealed by the existing source-identifier boundary inventory.
use crate::application_operation::ApplicationEncodedInput;
use crate::application_schema::{ApplicationStructuredValueBinding, ApplicationValueEncodeDenial};
/// Native backing must lend its one immutable stored value on every borrow.
/// Only the sealed ExecutionArray owner implements this integration contract.
pub trait NativeImmutableInput<Value> {
    fn value(&self) -> &Value;
}
pub fn encode_native_input<Binding, Owner>(
    owner: Owner,
) -> Result<ApplicationEncodedInput<Binding>, ApplicationValueEncodeDenial>
where
    Binding: ApplicationStructuredValueBinding,
    Binding::Value: serde::Serialize,
    Owner: NativeImmutableInput<Binding::Value> + Send + Sync + 'static,
{
    ApplicationEncodedInput::from_native_owned(owner)
}
