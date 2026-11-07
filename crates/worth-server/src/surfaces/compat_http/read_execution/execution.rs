use worth_proof::TransitionOutcome;

use crate::{
    WorthServerAdmission, WorthServerAdmittedDirectDeclaration,
    WorthServerCompatibilityPreparedRequest, WorthServerDirectDeclaration,
    WorthServerDirectDeclarationDenial, WorthServerQueryHandoffDeferred,
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
    WorthServerQueryHandoffFailure, WorthServerQueryHandoffRebindRequired,
    WorthServerQueryHandoffStale,
};

pub type WorthServerCompatibilityExecutionOutcome<T> = TransitionOutcome<
    T,
    WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDeferred,
    WorthServerQueryHandoffStale,
    WorthServerQueryHandoffRebindRequired,
    WorthServerQueryHandoffFailure,
>;

#[derive(Clone, Debug)]
pub struct WorthServerCompatibilityExecutionInput {
    prepared_request: WorthServerCompatibilityPreparedRequest,
    operation_name: String,
}

impl WorthServerCompatibilityExecutionInput {
    pub fn new(
        prepared_request: WorthServerCompatibilityPreparedRequest,
        operation_name: impl Into<String>,
    ) -> Self {
        Self {
            prepared_request,
            operation_name: operation_name.into().trim().to_string(),
        }
    }

    pub fn prepared_request(&self) -> &WorthServerCompatibilityPreparedRequest {
        &self.prepared_request
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub(crate) fn into_parts(self) -> (WorthServerCompatibilityPreparedRequest, String) {
        (self.prepared_request, self.operation_name)
    }
}

pub(crate) fn named_read_declaration(operation_name: &str) -> WorthServerDirectDeclaration {
    WorthServerDirectDeclaration::named_read(operation_name)
}

pub(crate) fn admit_declaration(
    intake: &crate::declaration_intake::WorthServerDirectDeclarationIntakeFacade,
    admission: WorthServerAdmission,
    declaration: WorthServerDirectDeclaration,
) -> Result<WorthServerAdmittedDirectDeclaration, WorthServerQueryHandoffDenial> {
    let prepared = intake
        .prepare(admission.clone(), declaration)
        .map_err(|denial| map_declaration_denial(&admission, denial))?;
    prepared
        .admit()
        .map_err(|denial| map_declaration_denial(&admission, denial))
}

fn map_declaration_denial(
    admission: &WorthServerAdmission,
    denial: WorthServerDirectDeclarationDenial,
) -> WorthServerQueryHandoffDenial {
    let diagnostics = admission.request_context().diagnostics_profile();
    match denial {
        WorthServerDirectDeclarationDenial::EvidenceIdentityDerivationDenied {
            cause,
            detail,
            ..
        } => WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause),
            diagnostics,
            detail,
        ),
        WorthServerDirectDeclarationDenial::InvalidDeclarationIdentity { detail, .. } => {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
                diagnostics,
                detail,
            )
        }
        WorthServerDirectDeclarationDenial::WorkspaceBindingFailed { detail, .. } => {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed,
                diagnostics,
                detail,
            )
        }
        WorthServerDirectDeclarationDenial::SourceNotAdmitted { detail, .. } => {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::DirectDeclarationSourceNotAdmitted,
                diagnostics,
                detail,
            )
        }
        WorthServerDirectDeclarationDenial::QueryFacadeFamilyNotAdmitted { detail, .. } => {
            WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily,
                diagnostics,
                detail,
            )
        }
    }
}
