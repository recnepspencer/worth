use worth_store_aspect_native::{
    StoreCanonicalBasisFamily, StoreDigestAuthority, StoreTerminalDocumentChecksum,
    StoreTerminalProjectionDocumentBytes,
};

fn main() {
    let document = StoreTerminalProjectionDocumentBytes::from_terminal_projection_bytes(
        br#"{"segment":"segment-0013"}"#.to_vec(),
    )
    .unwrap();
    let checksum = StoreTerminalDocumentChecksum::for_terminal_projection_document_bytes(&document);

    let _authority = StoreDigestAuthority::for_native_basis(
        StoreCanonicalBasisFamily::AspectBoundaryFact,
        checksum,
    );
}
