use crate::facade::{BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind};

use super::super::BridgeOwnedSignalRuntime;

impl BridgeOwnedSignalRuntime {
    pub fn validate_owned_async_request_occurrence<'a>(
        &self,
        request: &'a super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<&'a crate::facade::AdmittedBridgeAsyncRequestIdentity, BridgeAsyncCompletionRejection>
    {
        self.require_owned_async_request(request)?;
        Ok(request.request())
    }

    pub fn admit_owned_async_supersession<'a>(
        &self,
        prior: &'a super::super::BridgeOwnedAsyncRequestAdmission,
        displacing: &'a super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        super::super::BridgeOwnedAsyncSupersessionAdmission<'a>,
        BridgeAsyncCompletionRejection,
    > {
        self.require_owned_async_request(prior)?;
        self.require_owned_async_request(displacing)?;
        if prior.request().lowered().declaration_identity()
            != displacing.request().lowered().declaration_identity()
        {
            return Err(rejected(
                "owned async supersession must remain within one installed declaration",
            ));
        }
        if prior.request().request_handle() == displacing.request().request_handle() {
            return Err(rejected(
                "owned async supersession requires distinct admitted request occurrences",
            ));
        }
        Ok(super::super::BridgeOwnedAsyncSupersessionAdmission::new(
            prior, displacing,
        ))
    }
}

fn rejected(detail: &'static str) -> BridgeAsyncCompletionRejection {
    BridgeAsyncCompletionRejection::new(
        BridgeAsyncCompletionRejectionKind::SupersessionMismatch,
        detail,
    )
}
