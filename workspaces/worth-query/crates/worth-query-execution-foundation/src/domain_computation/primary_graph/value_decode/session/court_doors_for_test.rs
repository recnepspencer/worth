//! Sealed test doors over this module's private members.
use super::*;

impl<'a> ValueDecodeSession<'a> {
    #[doc(hidden)]
    pub fn denial_for_test(&self) -> Option<Stop> {
        Self::denial(self)
    }
}

impl<'a> ValueDecodeSession<'a> {
    #[doc(hidden)]
    pub fn checkpoint_for_test(&self) -> &Option<&'a dyn Fn(u64) -> Result<(), MapKernelStop>> {
        &self.checkpoint
    }
}

impl<'a> ValueDecodeSession<'a> {
    #[doc(hidden)]
    pub fn with_record_admission_for_test<Output>(
        &self,
        decode: impl FnOnce(
            &mut worth_query_declaration::facade::application_schema::ValueDecodeAdmission<'_>,
        ) -> Output,
    ) -> Result<Output, Stop> {
        Self::with_record_admission(self, decode)
    }
}
