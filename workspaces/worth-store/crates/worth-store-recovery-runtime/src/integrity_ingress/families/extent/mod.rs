mod chunk;
mod manifest;

pub(crate) use chunk::{admit_extent_chunk_projection, IntegrityAdmittedExtentChunkFrame};
pub(crate) use manifest::{admit_extent_manifest_projection, IntegrityAdmittedExtentManifest};

#[cfg(test)]
pub(super) fn owner_valid_compile_contracts() {
    manifest::owner_valid_compile_contract();
    chunk::owner_valid_compile_contract();
}
