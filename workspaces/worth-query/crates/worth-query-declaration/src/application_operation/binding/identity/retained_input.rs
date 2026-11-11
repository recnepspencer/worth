//! A canonical snapshot is mandatory before original input is erased.
use super::{
    ApplicationCanonicalInput, ApplicationEncodedInput, ApplicationRetainedInputDenial,
    ApplicationSharedInput,
};
use crate::application_schema::ApplicationStructuredValueBinding;

/// Immutable recorded value DATA held beside its original value custody.
/// Reconstruction reads the canonical stream; fresh admission governs its use.
pub struct ApplicationRetainedInput<Binding: ApplicationStructuredValueBinding> {
    encoded: ApplicationEncodedInput<Binding>,
    canonical: ApplicationCanonicalInput,
}
impl<Binding: ApplicationStructuredValueBinding> ApplicationEncodedInput<Binding>
where
    Binding::Value: serde::Serialize,
{
    /// Capture before erasure. An existing canonical capture is shared.
    /// If capture is first needed here, a changed live value refuses instead
    /// of replacing its original identity.
    pub fn retain_canonical_input(
        self,
    ) -> Result<ApplicationRetainedInput<Binding>, ApplicationRetainedInputDenial> {
        let canonical = self.capture_for_retention()?;
        Ok(ApplicationRetainedInput {
            encoded: self,
            canonical,
        })
    }
}
impl<Binding: ApplicationStructuredValueBinding> ApplicationRetainedInput<Binding> {
    pub const fn identity(&self) -> &[u8; 32] {
        self.encoded.identity()
    }
    /// Share the exact captured stream with its owning durable material publisher.
    pub const fn canonical_snapshot(&self) -> &ApplicationCanonicalInput {
        &self.canonical
    }
    pub fn recorded_bytes(&self) -> &[u8] {
        self.canonical.recorded_bytes()
    }
    /// Move the held value into original admission. This issues no authority.
    pub fn into_shared_input(self) -> ApplicationSharedInput<Binding::Value> {
        self.encoded.into_shared_input()
    }
}
impl<Binding: ApplicationStructuredValueBinding> Clone for ApplicationRetainedInput<Binding> {
    fn clone(&self) -> Self {
        Self {
            encoded: self.encoded.clone(),
            canonical: self.canonical.clone(),
        }
    }
}
impl<Binding: ApplicationStructuredValueBinding> std::fmt::Debug
    for ApplicationRetainedInput<Binding>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationRetainedInput")
            .field("identity", self.identity())
            .finish_non_exhaustive()
    }
}
