use worth_store_physical_integrity::{
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedPageFrame, UntrustedPhysicalArtifact,
};

fn consume_page(_: IntegrityValidatedPageFrame<'_>) {}
fn consume_extent(_: IntegrityValidatedExtentChunkFrame<'_>) {}

fn main() {
    let bytes = [];
    let untrusted = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    consume_page(untrusted);
    consume_extent(untrusted);
}
