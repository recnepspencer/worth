use crate::{
    WorthServerBinaryCertificationBundle, WorthServerCompatibilityFileEnvelope,
    WorthServerIngressIntegrityDigest, WorthServerIngressPerformanceReceipt,
    WorthServerMultipartUpload,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerCompatibilityUpload {
    upload: WorthServerMultipartUpload,
    ingress_integrity: WorthServerIngressIntegrityDigest,
    ingress_performance: WorthServerIngressPerformanceReceipt,
    mutation: crate::WorthServerCompatibilityMutation,
    file_envelope: WorthServerCompatibilityFileEnvelope,
    certification_bundle: WorthServerBinaryCertificationBundle,
    canonical_digest:
        Result<String, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>,
}

impl WorthServerCompatibilityUpload {
    pub(crate) fn new(
        upload: WorthServerMultipartUpload,
        ingress_integrity: WorthServerIngressIntegrityDigest,
        ingress_performance: WorthServerIngressPerformanceReceipt,
        mutation: crate::WorthServerCompatibilityMutation,
        file_envelope: WorthServerCompatibilityFileEnvelope,
        certification_bundle: WorthServerBinaryCertificationBundle,
    ) -> Self {
        let canonical_digest = mutation.canonical_digest().map(|mutation_digest| format!(
            "worth-server-compat-upload-v4|upload={}|integrity={}|ingress_performance={}|mutation={}|file_envelope={}|certification={}",
            upload.canonical_digest(),
            ingress_integrity.canonical_digest(),
            ingress_performance_digest(&ingress_performance),
            mutation_digest,
            file_envelope.canonical_digest(),
            certification_bundle.canonical_digest(),
        ));
        Self {
            upload,
            ingress_integrity,
            ingress_performance,
            mutation,
            file_envelope,
            certification_bundle,
            canonical_digest,
        }
    }

    pub fn upload(&self) -> &WorthServerMultipartUpload {
        &self.upload
    }

    pub fn ingress_integrity(&self) -> &WorthServerIngressIntegrityDigest {
        &self.ingress_integrity
    }

    pub fn ingress_performance(&self) -> &WorthServerIngressPerformanceReceipt {
        &self.ingress_performance
    }

    pub fn mutation(&self) -> &crate::WorthServerCompatibilityMutation {
        &self.mutation
    }

    pub fn file_envelope(&self) -> &WorthServerCompatibilityFileEnvelope {
        &self.file_envelope
    }

    pub fn certification_bundle(&self) -> &WorthServerBinaryCertificationBundle {
        &self.certification_bundle
    }

    pub fn canonical_digest(
        &self,
    ) -> Result<&str, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>
    {
        self.canonical_digest.as_deref().map_err(|cause| *cause)
    }
}

fn ingress_performance_digest(receipt: &WorthServerIngressPerformanceReceipt) -> String {
    receipt
        .receipt()
        .counter_rows()
        .iter()
        .map(|row| format!("{}={}", row.name().as_str(), row.observed_count()))
        .collect::<Vec<_>>()
        .join("|")
}
