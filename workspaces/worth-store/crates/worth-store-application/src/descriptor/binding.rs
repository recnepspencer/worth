//! Canonical descriptive program/roster bytes. They grant no installation authority.
use worth_foundational::facade::ApplicationHomeBinding;
pub(super) mod readmission;
pub(crate) use readmission::{allocation_bytes, decode, describes};

pub(crate) fn size(binding: &ApplicationHomeBinding) -> Option<usize> {
    binding.roster().iter().try_fold(
        64_usize
            .checked_add(4)?
            .checked_add(program_size(binding.selected())?)?,
        |size, program| size.checked_add(program_size(program)?),
    )
}

fn program_size(program: &worth_foundational::facade::HomeProgramBinding) -> Option<usize> {
    36_usize.checked_add(program.identity().len())
}

pub(crate) fn emit(binding: &ApplicationHomeBinding, bytes: &mut Vec<u8>) {
    emit_parts(binding, &mut |part| bytes.extend_from_slice(part));
}

pub(crate) fn commitment(binding: &ApplicationHomeBinding) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    emit_parts(binding, &mut |part| digest.update(part));
    digest.finalize().into()
}

fn emit_parts(binding: &ApplicationHomeBinding, out: &mut dyn FnMut(&[u8])) {
    out(binding.package().bytes());
    out(binding.schema().bytes());
    program(binding.selected(), out);
    out(&(binding.roster().len() as u32).to_le_bytes());
    for revision in binding.roster() {
        program(revision, out);
    }
}

fn program(program: &worth_foundational::facade::HomeProgramBinding, out: &mut dyn FnMut(&[u8])) {
    out(&(program.identity().len() as u32).to_le_bytes());
    out(program.identity().as_bytes());
    out(program.revision().bytes());
}
