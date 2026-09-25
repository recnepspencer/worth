use sha2::{Digest, Sha256};

use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

pub(super) fn output_content_identity(
    receipt: &WorthQueryApplicationCommitReceipt,
    source_identity: [u8; 32],
) -> String {
    let publication = receipt.committed_product_publication();
    let mut digest = Sha256::new();
    digest.update(b"worth-query:workflow-assessment-output-content:v1");
    digest.update(receipt.output_correspondence().workflow_content_identity());
    digest.update(source_identity);
    digest.update(receipt.installed_operation());
    digest.update(
        publication
            .product_branch()
            .owner_identity()
            .get()
            .to_le_bytes(),
    );
    digest.update(publication.product_branch().name().as_str().as_bytes());
    digest.update(publication.product_incarnation().ordinal().to_le_bytes());
    digest.update(publication.composite_commit().ordinal().to_le_bytes());
    digest.update(publication.relational_commit().commit_id.0.to_le_bytes());
    digest.update(publication.relational_commit().version_id.0.to_le_bytes());
    hex(digest.finalize().into())
}

pub(super) fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;

    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

pub(super) fn decode_hex_identity(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&text[offset..offset + 2], 16).ok()?;
    }
    Some(bytes)
}
