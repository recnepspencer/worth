use crate::{
    WorthServerOperationFamily, WorthServerOperationRequest, WorthServerProductOperationBasisKind,
};

use super::{
    WorthServerProductSession, WorthServerProductSessionExpiryPosture,
    WorthServerProductSessionLifecycle,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductSessionDenialCode {
    CoordinationRequestDenied,
    CoordinationAdmissionDenied,
    CoordinationReadinessDenied,
    MissingProductSessionIdentity,
    UnknownProductSessionIdentity,
    ForeignProductSession,
    ExpiredProductSession,
    ClosedProductSession,
    PreviewSessionCannotMutate,
    SessionRebindRequired,
}

impl WorthServerProductSessionDenialCode {
    pub fn reason_key(self) -> &'static str {
        match self {
            Self::CoordinationRequestDenied => "worth_product_session_coordination_request_denied",
            Self::CoordinationAdmissionDenied => {
                "worth_product_session_coordination_admission_denied"
            }
            Self::CoordinationReadinessDenied => {
                "worth_product_session_coordination_readiness_denied"
            }
            Self::MissingProductSessionIdentity => "worth_product_session_identity_missing",
            Self::UnknownProductSessionIdentity => "worth_product_session_unknown",
            Self::ForeignProductSession => "worth_product_session_foreign",
            Self::ExpiredProductSession => "worth_product_session_expired",
            Self::ClosedProductSession => "worth_product_session_closed",
            Self::PreviewSessionCannotMutate => "worth_product_session_preview_mutation_denied",
            Self::SessionRebindRequired => "worth_product_session_rebind_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductSessionDenial {
    code: WorthServerProductSessionDenialCode,
    detail: String,
}

impl WorthServerProductSessionDenial {
    pub(crate) fn new(
        code: WorthServerProductSessionDenialCode,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub fn code(&self) -> WorthServerProductSessionDenialCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub(crate) fn admit_session_for_product_operation(
    session: Option<&WorthServerProductSession>,
    request: &WorthServerOperationRequest,
    basis_kind: WorthServerProductOperationBasisKind,
    requires_mutation_session: bool,
) -> Result<(), WorthServerProductSessionDenial> {
    let requires_session = requires_mutation_session
        || basis_kind == WorthServerProductOperationBasisKind::ProductSessionDerived;
    if !requires_session {
        return Ok(());
    }
    let requested_identity = request
        .identity()
        .product_session_identity()
        .ok_or_else(|| {
            WorthServerProductSessionDenial::new(
                WorthServerProductSessionDenialCode::MissingProductSessionIdentity,
                "product operation requires an admitted product session identity",
            )
        })?;
    let session = session.ok_or_else(|| {
        WorthServerProductSessionDenial::new(
            WorthServerProductSessionDenialCode::UnknownProductSessionIdentity,
            format!("product session `{requested_identity}` was not found"),
        )
    })?;
    validate_session_scope(session, request)?;
    validate_session_branch_and_basis(session, request, basis_kind)?;
    validate_session_lifecycle(session, request)?;
    Ok(())
}

fn validate_session_scope(
    session: &WorthServerProductSession,
    request: &WorthServerOperationRequest,
) -> Result<(), WorthServerProductSessionDenial> {
    let workspace_target = request
        .resolved_request_context()
        .request_context()
        .workspace_target();
    if session.tenant_id() != workspace_target.tenant_id()
        || session.workspace_id() != workspace_target.workspace_id()
    {
        return Err(WorthServerProductSessionDenial::new(
            WorthServerProductSessionDenialCode::ForeignProductSession,
            format!(
                "product session `{}` does not belong to tenant `{}` workspace `{}`",
                session.identity().as_str(),
                workspace_target.tenant_id(),
                workspace_target.workspace_id()
            ),
        ));
    }
    Ok(())
}

fn validate_session_branch_and_basis(
    session: &WorthServerProductSession,
    request: &WorthServerOperationRequest,
    basis_kind: WorthServerProductOperationBasisKind,
) -> Result<(), WorthServerProductSessionDenial> {
    let branch_label = request
        .resolved_request_context()
        .request_context()
        .branch_target()
        .canonical_label();
    if session.branch_label() != branch_label {
        return Err(WorthServerProductSessionDenial::new(
            WorthServerProductSessionDenialCode::SessionRebindRequired,
            format!(
                "product session `{}` is bound to branch `{}` and cannot be used on `{}` without explicit rebind",
                session.identity().as_str(),
                session.branch_label(),
                branch_label
            ),
        ));
    }
    if basis_kind == WorthServerProductOperationBasisKind::QueryDerived
        && let (Some(request_basis_digest), Some(session_basis_digest)) =
            (request.identity().basis_digest(), session.basis_digest())
        && request_basis_digest != session_basis_digest
    {
        return Err(WorthServerProductSessionDenial::new(
            WorthServerProductSessionDenialCode::SessionRebindRequired,
            format!(
                "product session `{}` is bound to basis `{}` and cannot authorize `{}` without explicit rebind",
                session.identity().as_str(),
                session_basis_digest,
                request_basis_digest
            ),
        ));
    }
    Ok(())
}

fn validate_session_lifecycle(
    session: &WorthServerProductSession,
    request: &WorthServerOperationRequest,
) -> Result<(), WorthServerProductSessionDenial> {
    match session.expiry_posture() {
        WorthServerProductSessionExpiryPosture::Expired { .. } => {
            return Err(WorthServerProductSessionDenial::new(
                WorthServerProductSessionDenialCode::ExpiredProductSession,
                format!(
                    "product session `{}` has expired",
                    session.identity().as_str()
                ),
            ));
        }
        WorthServerProductSessionExpiryPosture::Closed { .. } => {
            return Err(WorthServerProductSessionDenial::new(
                WorthServerProductSessionDenialCode::ClosedProductSession,
                format!(
                    "product session `{}` has been closed",
                    session.identity().as_str()
                ),
            ));
        }
        WorthServerProductSessionExpiryPosture::Active { .. } => {}
    }
    if request.identity().operation_family()
        == WorthServerOperationFamily::ProductApplicationMutation
        && session.lifecycle() == WorthServerProductSessionLifecycle::ReadOnlyPreview
    {
        return Err(WorthServerProductSessionDenial::new(
            WorthServerProductSessionDenialCode::PreviewSessionCannotMutate,
            format!(
                "product session `{}` is preview-only and cannot authorize mutation work",
                session.identity().as_str()
            ),
        ));
    }
    Ok(())
}
