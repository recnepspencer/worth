use super::super::{
    WorthServerBinaryPolicyDecisionParts, WorthServerFileMetadataReceiptParts,
    WorthServerFileTransferProvenanceParts,
};
use crate::{
    WorthServerBinaryPolicyDecision, WorthServerCacheabilityPolicy, WorthServerCanonicalFilename,
    WorthServerCompatibilityFileEnvelope, WorthServerFileMetadataReceipt,
    WorthServerFileMetadataTruthKind, WorthServerFileTransferDisposition,
    WorthServerFileTransferProvenance, WorthServerMetadataNormalizationReceipt,
};

pub(crate) fn project_metadata_read_envelope(
    direct_context: &crate::WorthServerDirectContextArtifact,
    operation_name: &str,
    truth_digest: &str,
    response_envelope: &crate::WorthServerResponseEnvelope,
    support_posture: &crate::WorthServerQuerySupportPosture,
    compatibility_cache_policy: &crate::WorthServerCompatibilityCachePolicy,
) -> WorthServerCompatibilityFileEnvelope {
    project_observed_envelope(ObservedEnvelopeProjection {
        direct_context,
        operation_name,
        truth_kind: WorthServerFileMetadataTruthKind::ObservedRead,
        truth_digest,
        response_envelope,
        support_posture,
        surface_kind: "metadata_read",
        transfer_disposition: WorthServerFileTransferDisposition::MetadataOnlyObservation,
        compatibility_cache_policy,
        content_type: None,
        bytes_selected: 0,
        range_honored: false,
    })
}

pub(crate) fn project_metadata_inspection_envelope(
    direct_context: &crate::WorthServerDirectContextArtifact,
    operation_name: &str,
    truth_digest: &str,
    response_envelope: &crate::WorthServerResponseEnvelope,
    support_posture: &crate::WorthServerQuerySupportPosture,
    compatibility_cache_policy: &crate::WorthServerCompatibilityCachePolicy,
) -> WorthServerCompatibilityFileEnvelope {
    project_observed_envelope(ObservedEnvelopeProjection {
        direct_context,
        operation_name,
        truth_kind: WorthServerFileMetadataTruthKind::ObservedInspection,
        truth_digest,
        response_envelope,
        support_posture,
        surface_kind: "metadata_inspection",
        transfer_disposition: WorthServerFileTransferDisposition::MetadataOnlyObservation,
        compatibility_cache_policy,
        content_type: None,
        bytes_selected: 0,
        range_honored: false,
    })
}

pub(crate) fn project_binary_egress_envelope(
    read: &crate::WorthServerCompatibilityRead,
    content_type: Option<String>,
    bytes_selected: u64,
    range_honored: bool,
    disposition: WorthServerFileTransferDisposition,
) -> WorthServerCompatibilityFileEnvelope {
    project_observed_envelope(ObservedEnvelopeProjection {
        direct_context: read.direct_context(),
        operation_name: read.operation_name(),
        truth_kind: WorthServerFileMetadataTruthKind::ObservedRead,
        truth_digest: read.read_result().receipt().result_digest(),
        response_envelope: read.response_envelope(),
        support_posture: read.support_posture(),
        surface_kind: "binary_egress",
        transfer_disposition: disposition,
        compatibility_cache_policy: read.cache_policy(),
        content_type,
        bytes_selected,
        range_honored,
    })
}

pub(crate) fn project_upload_envelope(
    session: &crate::WorthServerBinaryIngressSession,
    mutation: &crate::WorthServerCompatibilityMutation,
) -> Result<
    WorthServerCompatibilityFileEnvelope,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    let canonical_filename = WorthServerCanonicalFilename::admit(
        session.operation_name(),
        mutation
            .envelope()
            .response_envelope()
            .diagnostics_profile(),
        crate::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
    )
    .expect("upload file identity should already be validated before truth linkage");
    let metadata_receipt =
        WorthServerFileMetadataReceipt::new(WorthServerFileMetadataReceiptParts {
            tenant_id: session.tenant_id().to_string(),
            workspace_digest: session.workspace_digest().to_string(),
            branch_digest: session.branch_digest().to_string(),
            operation_name: canonical_filename.canonical().to_string(),
            truth_kind: WorthServerFileMetadataTruthKind::CommittedMutation,
            truth_digest: mutation.mutation_result().result_digest()?.to_string(),
            basis_digest: mutation
                .envelope()
                .direct_context()
                .basis_digest()
                .map(str::to_string),
            provenance: mutation
                .envelope()
                .direct_context()
                .provenance()
                .artifact()
                .clone(),
        });
    let metadata_normalization_receipt = WorthServerMetadataNormalizationReceipt::from_manifest(
        metadata_receipt.metadata_identity(),
        canonical_filename.canonical(),
        session.upload().manifest().metadata_body(),
        mutation
            .envelope()
            .response_envelope()
            .diagnostics_profile(),
        crate::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
    )
    .expect("upload metadata normalization should already be validated before truth linkage");
    let policy_decision =
        WorthServerBinaryPolicyDecision::new(WorthServerBinaryPolicyDecisionParts {
            metadata_identity: metadata_receipt.metadata_identity().to_string(),
            tenant_id: metadata_receipt.tenant_id().to_string(),
            workspace_digest: metadata_receipt.workspace_digest().to_string(),
            branch_digest: metadata_receipt.branch_digest().to_string(),
            operation_name: metadata_receipt.operation_name().to_string(),
            diagnostics_profile: mutation
                .envelope()
                .response_envelope()
                .diagnostics_profile(),
            policy_lane: "upload_ingress".to_string(),
            support_posture_digest: mutation.envelope().support_posture().canonical_label(),
            response_envelope_digest: mutation
                .envelope()
                .response_envelope()
                .canonical_digest()
                .to_string(),
            transfer_authorization_digest: None,
        });
    let transfer_provenance =
        WorthServerFileTransferProvenance::new(WorthServerFileTransferProvenanceParts {
            metadata_identity: metadata_receipt.metadata_identity().to_string(),
            tenant_id: metadata_receipt.tenant_id().to_string(),
            workspace_digest: metadata_receipt.workspace_digest().to_string(),
            branch_digest: metadata_receipt.branch_digest().to_string(),
            operation_name: metadata_receipt.operation_name().to_string(),
            diagnostics_profile: mutation
                .envelope()
                .response_envelope()
                .diagnostics_profile(),
            disposition: WorthServerFileTransferDisposition::VerifiedIngress,
            content_type: None,
            bytes_selected: session
                .upload()
                .parts()
                .iter()
                .map(|part| part.authoritative_len())
                .sum(),
            range_honored: false,
        });
    let cacheability_policy = WorthServerCacheabilityPolicy::scoped_private(
        "upload_ingress",
        mutation
            .envelope()
            .response_envelope()
            .diagnostics_profile(),
        metadata_receipt.metadata_identity(),
        metadata_receipt.branch_digest(),
    );
    Ok(WorthServerCompatibilityFileEnvelope::new(
        metadata_receipt,
        canonical_filename,
        metadata_normalization_receipt,
        cacheability_policy,
        policy_decision,
        transfer_provenance,
    ))
}

struct ObservedEnvelopeProjection<'a> {
    direct_context: &'a crate::WorthServerDirectContextArtifact,
    operation_name: &'a str,
    truth_kind: WorthServerFileMetadataTruthKind,
    truth_digest: &'a str,
    response_envelope: &'a crate::WorthServerResponseEnvelope,
    support_posture: &'a crate::WorthServerQuerySupportPosture,
    surface_kind: &'a str,
    transfer_disposition: WorthServerFileTransferDisposition,
    compatibility_cache_policy: &'a crate::WorthServerCompatibilityCachePolicy,
    content_type: Option<String>,
    bytes_selected: u64,
    range_honored: bool,
}

fn project_observed_envelope(
    parts: ObservedEnvelopeProjection<'_>,
) -> WorthServerCompatibilityFileEnvelope {
    let ObservedEnvelopeProjection {
        direct_context,
        operation_name,
        truth_kind,
        truth_digest,
        response_envelope,
        support_posture,
        surface_kind,
        transfer_disposition,
        compatibility_cache_policy,
        content_type,
        bytes_selected,
        range_honored,
    } = parts;
    let canonical_filename = WorthServerCanonicalFilename::admit(
        operation_name,
        response_envelope.diagnostics_profile(),
        crate::WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
    )
    .expect("observed file identity should already be validated before projection");
    let metadata_receipt =
        WorthServerFileMetadataReceipt::new(WorthServerFileMetadataReceiptParts {
            tenant_id: direct_context.workspace_target().tenant_id().to_string(),
            workspace_digest: direct_context.workspace_digest().to_string(),
            branch_digest: direct_context.branch_digest().to_string(),
            operation_name: canonical_filename.canonical().to_string(),
            truth_kind,
            truth_digest: truth_digest.to_string(),
            basis_digest: direct_context.basis_digest().map(str::to_string),
            provenance: direct_context.provenance().artifact().clone(),
        });
    let metadata_normalization_receipt = WorthServerMetadataNormalizationReceipt::observed(
        metadata_receipt.metadata_identity(),
        canonical_filename.canonical(),
        response_envelope.diagnostics_profile(),
    );
    let policy_decision =
        WorthServerBinaryPolicyDecision::new(WorthServerBinaryPolicyDecisionParts {
            metadata_identity: metadata_receipt.metadata_identity().to_string(),
            tenant_id: metadata_receipt.tenant_id().to_string(),
            workspace_digest: metadata_receipt.workspace_digest().to_string(),
            branch_digest: metadata_receipt.branch_digest().to_string(),
            operation_name: metadata_receipt.operation_name().to_string(),
            diagnostics_profile: response_envelope.diagnostics_profile(),
            policy_lane: surface_kind.to_string(),
            support_posture_digest: support_posture.canonical_label(),
            response_envelope_digest: response_envelope.canonical_digest().to_string(),
            transfer_authorization_digest: None,
        });
    let transfer_provenance =
        WorthServerFileTransferProvenance::new(WorthServerFileTransferProvenanceParts {
            metadata_identity: metadata_receipt.metadata_identity().to_string(),
            tenant_id: metadata_receipt.tenant_id().to_string(),
            workspace_digest: metadata_receipt.workspace_digest().to_string(),
            branch_digest: metadata_receipt.branch_digest().to_string(),
            operation_name: metadata_receipt.operation_name().to_string(),
            diagnostics_profile: response_envelope.diagnostics_profile(),
            disposition: transfer_disposition,
            content_type,
            bytes_selected,
            range_honored,
        });
    let cacheability_policy = WorthServerCacheabilityPolicy::from_compatibility_policy(
        surface_kind,
        response_envelope.diagnostics_profile(),
        metadata_receipt.metadata_identity(),
        metadata_receipt.branch_digest(),
        compatibility_cache_policy,
        false,
    );
    WorthServerCompatibilityFileEnvelope::new(
        metadata_receipt,
        canonical_filename,
        metadata_normalization_receipt,
        cacheability_policy,
        policy_decision,
        transfer_provenance,
    )
}
