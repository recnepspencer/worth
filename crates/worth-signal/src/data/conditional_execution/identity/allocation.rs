use super::{InstalledSignalConditionalContract, SignalConditionalDependencyVersion};
use crate::data::conditional_execution::conditional_work;
use crate::data::error::SignalError;
use crate::data::retained_storage::RetainedStoragePreparation;

pub(super) fn admit(
    contract: &InstalledSignalConditionalContract,
    snapshot: &str,
    execution: &str,
    dependencies: &[SignalConditionalDependencyVersion],
    work: &mut RetainedStoragePreparation,
) -> Result<usize, SignalError> {
    // Each fixed envelope/edge has fewer than 512 bytes of labels, delimiters,
    // decimal lengths, scalar values and enum tags (usize/u64 use <=20 digits).
    // Variable text is added separately, including both scope strings. No
    // renderer or allocator runs during this admitted metadata pass.
    conditional_work::reserve(work, dependencies.len().checked_add(1))?;
    let mut bytes = conditional_work::checked(
        work,
        dependencies
            .len()
            .checked_add(1)
            .and_then(|count| count.checked_mul(512))
            .and_then(|bytes| bytes.checked_add(contract.projection_contract().len()))
            .and_then(|bytes| bytes.checked_add(snapshot.len()))
            .and_then(|bytes| bytes.checked_add(execution.len())),
    )?;
    for dependency in dependencies {
        if let Some(scope) = &dependency.scope {
            bytes = conditional_work::checked(
                work,
                bytes
                    .checked_add(scope.partition.0.len())
                    .and_then(|bytes| {
                        bytes.checked_add(scope.detail.as_ref().map_or(0, String::len))
                    }),
            )?;
        }
    }
    conditional_work::checked(work, (bytes <= isize::MAX as usize).then_some(bytes))?;
    // Rendering and scalar formatting, then String -> Arc<str> identity storage.
    // The String receives the whole admitted capacity, so it never grows.
    conditional_work::reserve(work, bytes.checked_mul(3))?;
    Ok(bytes)
}
