mod chunk;
mod manifest;

pub(crate) use chunk::{
    admit_extent_chunk_projection, ExtentChunkProjection, IntegrityAdmittedExtentChunkFrame,
};
pub(crate) use manifest::{
    admit_extent_manifest_projection, AdmittedRecoveryExtentManifest, ExtentManifestProjection,
    IntegrityAdmittedExtentManifest,
};

#[cfg(test)]
pub(super) fn owner_valid_compile_contracts() {
    manifest::owner_valid_compile_contract();
    chunk::owner_valid_compile_contract();
}
