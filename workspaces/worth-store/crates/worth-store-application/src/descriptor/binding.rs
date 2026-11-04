//! Canonical descriptive program/roster bytes. They grant no installation authority.
use worth_foundational::facade::ApplicationHomeBinding;
mod readmission;
pub(super) use readmission::{allocation_bytes, decode};

pub(super) fn size(binding: &ApplicationHomeBinding) -> Option<usize> {
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

pub(super) fn emit(binding: &ApplicationHomeBinding, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(binding.package().bytes());
    bytes.extend_from_slice(binding.schema().bytes());
    program(binding.selected(), bytes);
    bytes.extend_from_slice(&(binding.roster().len() as u32).to_le_bytes());
    for revision in binding.roster() {
        program(revision, bytes);
    }
}

fn program(program: &worth_foundational::facade::HomeProgramBinding, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(program.identity().len() as u32).to_le_bytes());
    bytes.extend_from_slice(program.identity().as_bytes());
    bytes.extend_from_slice(program.revision().bytes());
}
