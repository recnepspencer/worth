use std::collections::BTreeMap;

use crate::{
    WorthServerMultipartUpload, WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
};

use super::super::binary_digest::stable_byte_digest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerIngressIntegrityDigest {
    manifest_digest: String,
    part_digests: BTreeMap<String, String>,
    canonical_digest: String,
}

impl WorthServerIngressIntegrityDigest {
    pub(crate) fn verify(
        upload: &WorthServerMultipartUpload,
        diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
    ) -> Result<Self, WorthServerQueryHandoffDenial> {
        let manifest_digest = stable_byte_digest(upload.manifest().integrity_basis().as_bytes());
        ensure_declared_digest_matches(
            upload.manifest().declared_integrity_digest(),
            &manifest_digest,
            diagnostics_profile,
            "compatibility upload manifest",
        )?;

        let mut part_digests = BTreeMap::new();
        for part in upload.parts() {
            let digest = stable_byte_digest(&part.effective_authoritative_bytes());
            ensure_declared_digest_matches(
                part.declared_integrity_digest(),
                &digest,
                diagnostics_profile,
                &format!("compatibility upload part `{}`", part.name()),
            )?;
            part_digests.insert(part.name().trim().to_string(), digest);
        }

        let canonical_digest = format!(
            "worth-server-ingress-integrity-v1|manifest={}|parts={}",
            manifest_digest,
            part_digests
                .iter()
                .map(|(name, digest)| format!("{name}:{digest}"))
                .collect::<Vec<_>>()
                .join("|"),
        );
        Ok(Self {
            manifest_digest,
            part_digests,
            canonical_digest,
        })
    }

    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub fn part_digest(&self, name: &str) -> Option<&str> {
        self.part_digests.get(name).map(String::as_str)
    }

    pub fn part_digests(&self) -> &BTreeMap<String, String> {
        &self.part_digests
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

fn ensure_declared_digest_matches(
    declared_digest: Option<&str>,
    observed_digest: &str,
    diagnostics_profile: worth_foundational::facade::DiagnosticRichnessProfile,
    label: &str,
) -> Result<(), WorthServerQueryHandoffDenial> {
    if let Some(declared) = declared_digest
        && declared != observed_digest
    {
        return Err(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
            diagnostics_profile,
            format!(
                "{label} integrity digest mismatch: declared `{declared}` but observed `{observed_digest}`"
            ),
        ));
    }
    Ok(())
}
