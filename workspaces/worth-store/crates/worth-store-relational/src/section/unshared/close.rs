//! Unshared pre-service owners use the same original source retirement port.
use super::*;
impl UnrecoveredRelationalSection {
    pub fn prepare_close(
        self,
        preparation: super::super::close::PreparedRelationalSectionClose,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ClosingRelationalSection,
        (
            Self,
            super::super::close::PreparedRelationalSectionClose,
            RelationalSectionCloseDenial,
        ),
    > {
        let Self { attached, prepared_source } = self;
        match super::super::close::prepare_attached_close(attached, None, preparation, request)
        {
            Ok(closing) => Ok(closing),
            Err((attached, source, preparation, cause)) => {
                assert!(
                    source.is_none(),
                    "unrecovered owner never acquired source authority"
                );
                Err((Self { attached, prepared_source }, preparation, cause))
            }
        }
    }
}
impl UnsharedReadmittedRelationalSection {
    pub fn prepare_close(
        self,
        preparation: super::super::close::PreparedRelationalSectionClose,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ClosingRelationalSection,
        (
            Self,
            super::super::close::PreparedRelationalSectionClose,
            RelationalSectionCloseDenial,
        ),
    > {
        match super::super::close::prepare_attached_close(
            self.attached,
            Some(self.source),
            preparation,
            request,
        ) {
            Ok(closing) => Ok(closing),
            Err((attached, source, preparation, cause)) => Err((
                Self {
                    attached,
                    source: source.expect("same readmitted authority survives refusal"),
                },
                preparation,
                cause,
            )),
        }
    }
}
