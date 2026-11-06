//! Stream the unchanged carrier encoding into its digest without a staging Vec.
use super::SchemaImageData;
use serde::Serialize;
use sha2::{Digest, Sha256};

struct DigestWriter(Sha256);
impl std::io::Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl SchemaImageData {
    pub(super) fn recompute_digest(&self) -> Result<[u8; 32], rmp_serde::encode::Error> {
        let mut output = DigestWriter(Sha256::new());
        output
            .0
            .update(b"worth.relational.branch-root-schema-image.v1\0");
        (
            self.format_version,
            self.schema_root,
            self.descriptor_semantics_version,
            &self.entity_registrations,
            &self.relation_registrations,
            &self.retained_aspect_contracts,
        )
            .serialize(&mut rmp_serde::Serializer::new(&mut output))?;
        crate::derived_relation::canonical::update_digest(&mut output.0, &self.derived_relations);
        crate::projected_member_domain::canonical::update_digest(
            &mut output.0,
            &self.projected_member_domains,
        );
        output.0.update(b"worth.native.content-schema-carrier.v1\0");
        self.content_formats
            .serialize(&mut rmp_serde::Serializer::new(&mut output))?;
        Ok(output.0.finalize().into())
    }
}
