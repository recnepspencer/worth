use std::marker::PhantomData;
use std::sync::atomic::Ordering;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_runtime_bridge::facade::{
    BridgePreviewRetainedArtifactSchema, BridgePreviewSessionBasis,
    BridgePreviewSessionDeclaration, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, BridgeRequestKind, BridgeSignalBranchIdentity,
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeSpeculationError,
    BridgeSpeculativeBranchBinding, BridgeSpeculativeBranchBindingIdentity,
    BridgeSpeculativeSessionRequest, BridgeTruthViewSelector,
};

use super::{WorthQueryApplicationPreviewSession, WorthQueryApplicationPreviewSessionIdentity};
use crate::domain_computation::primary_graph::{
    application_branch::primary_truth_branch_identity, WorthQueryPrimaryGraphApplicationRuntime,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationPreviewSessionDenialKind {
    Cancelled,
    DeadlineExceeded,
    CurrentTruthUnavailable,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    SessionIdentityExhausted,
    BridgeRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationPreviewSessionDenial {
    kind: WorthQueryApplicationPreviewSessionDenialKind,
    subject: String,
}

impl WorthQueryApplicationPreviewSessionDenial {
    fn new(
        kind: WorthQueryApplicationPreviewSessionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationPreviewSessionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryApplicationPreviewSessionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.subject)
    }
}

impl std::error::Error for WorthQueryApplicationPreviewSessionDenial {}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn open_application_preview_session(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationPreviewSession<Schema>,
        WorthQueryApplicationPreviewSessionDenial,
    > {
        validate_request(request)?;
        let (identity, bridge_request, source_basis, source_observation) =
            self.current_preview_request()?;
        let handle = self
            .bridge
            .ordinary()
            .speculate(bridge_request)
            .map_err(bridge_denial)?;
        if let Err(denial) = validate_request(request) {
            let _ = handle.discard(Vec::new());
            return Err(denial);
        }
        Ok(WorthQueryApplicationPreviewSession {
            runtime_authority: self.runtime.authority_identity(),
            schema_binding: self.installed_schema.binding_identity(),
            identity,
            handle: Some(handle),
            source_basis,
            source_observation: Some(source_observation),
            _schema: PhantomData,
        })
    }

    fn current_preview_request(
        &self,
    ) -> Result<
        (
            WorthQueryApplicationPreviewSessionIdentity,
            BridgeSpeculativeSessionRequest,
            worth_relational::facade::branch::AdmittedRelationalBranchBasis,
            worth_relational::facade::bridge::RelationalBridgeObservationLease,
        ),
        WorthQueryApplicationPreviewSessionDenial,
    > {
        let (_, source_basis) = self
            .product_runtime
            .source
            .observe_branch_basis(&self.relational_branch_identity)
            .map_err(current_truth_denial)?;
        let source_observation = self
            .product_runtime
            .source
            .retain_branch_basis_for_bridge(&source_basis)
            .map_err(current_truth_denial)?;
        let sequence = next_preview_sequence(&self.next_preview_session)?;
        let identity_basis = format!("{}-{sequence}", self.runtime.authority_identity().as_u64());
        let identity = WorthQueryApplicationPreviewSessionIdentity::mint(format!(
            "worth-query-application-preview-session:{identity_basis}"
        ));
        let truth_branch = primary_truth_branch_identity();
        let declaration = BridgePreviewSessionDeclaration::new(
            BridgePreviewSessionDeclarationIdentity::from_stable_name(format!(
                "worth-query-application-preview-declaration:{identity_basis}"
            )),
            BridgeRequestKind::Preview,
            BridgeSpeculativeBranchBinding::new(
                BridgeSpeculativeBranchBindingIdentity::from_stable_name(format!(
                    "worth-query-application-preview-binding:{identity_basis}"
                )),
                truth_branch.clone(),
                BridgeSignalBranchIdentity::from_stable_name(format!(
                    "worth-query-application-preview-signal:{identity_basis}"
                )),
            ),
            BridgePreviewSessionBasis::new(
                BridgeTruthViewSelector::branch_snapshot(
                    truth_branch,
                    source_observation.snapshot_identity().clone(),
                ),
                BridgeSourceCapabilitySet::new(vec![
                    BridgeSourceCapability::SnapshotRead,
                    BridgeSourceCapability::BranchRead,
                ]),
                BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
            ),
        );
        Ok((
            identity.clone(),
            BridgeSpeculativeSessionRequest::new(
                BridgePreviewSessionIdentity::from_stable_name(identity.as_str()),
                declaration,
                1,
                1,
                0,
            ),
            source_basis,
            source_observation,
        ))
    }
}

fn next_preview_sequence(
    sequence: &std::sync::atomic::AtomicU64,
) -> Result<u64, WorthQueryApplicationPreviewSessionDenial> {
    sequence
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| {
            WorthQueryApplicationPreviewSessionDenial::new(
                WorthQueryApplicationPreviewSessionDenialKind::SessionIdentityExhausted,
                "application preview session identity space",
            )
        })
}

fn validate_request(
    request: &WorthQueryRequestScope,
) -> Result<(), WorthQueryApplicationPreviewSessionDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(WorthQueryApplicationPreviewSessionDenial::new(
                WorthQueryApplicationPreviewSessionDenialKind::Cancelled,
                "application preview session",
            ))
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(WorthQueryApplicationPreviewSessionDenial::new(
                WorthQueryApplicationPreviewSessionDenialKind::DeadlineExceeded,
                "application preview session",
            ))
        }
        None => Ok(()),
    }
}

fn current_truth_denial(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryApplicationPreviewSessionDenial {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryApplicationPreviewSessionDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryApplicationPreviewSessionDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryApplicationPreviewSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryApplicationPreviewSessionDenialKind::CurrentTruthUnavailable,
    };
    WorthQueryApplicationPreviewSessionDenial::new(
        kind,
        format!("primary application branch truth admission denied: {denial:?}"),
    )
}

pub(super) fn bridge_denial(
    error: BridgeSpeculationError,
) -> WorthQueryApplicationPreviewSessionDenial {
    WorthQueryApplicationPreviewSessionDenial::new(
        WorthQueryApplicationPreviewSessionDenialKind::BridgeRejected,
        error.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::{next_preview_sequence, WorthQueryApplicationPreviewSessionDenialKind};

    #[test]
    fn preview_session_identity_sequence_never_wraps() {
        let sequence = AtomicU64::new(u64::MAX);

        let denial = next_preview_sequence(&sequence)
            .expect_err("exhausted preview identity space must fail closed");

        assert_eq!(
            denial.kind(),
            WorthQueryApplicationPreviewSessionDenialKind::SessionIdentityExhausted
        );
        assert_eq!(
            sequence.load(std::sync::atomic::Ordering::Relaxed),
            u64::MAX
        );
    }
}
