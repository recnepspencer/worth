use super::WorthQueryApplicationAttemptDenial;
use worth_execution::{ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder};

/// Moves only the known inline payload into fresh admitted backing. Upstream
/// maps and nested values remain separate allocation owners.
pub(in crate::domain_computation::primary_graph::application_attempt) fn admit_array<T>(
    count: usize,
    values: impl IntoIterator<Item = T>,
    policy: ExecutionAllocationPolicy<'_, '_>,
    subject: &str,
    mut check_authority: impl FnMut() -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<ExecutionArray<T>, WorthQueryApplicationAttemptDenial> {
    check_authority()?;
    let mut array = ExecutionArrayBuilder::allocate(count, policy)
        .map_err(|denial| WorthQueryApplicationAttemptDenial::allocation_denied(subject, denial))?;
    for value in values {
        check_authority()?;
        array.push(value).map_err(|denial| {
            WorthQueryApplicationAttemptDenial::allocation_denied(subject, denial)
        })?;
    }
    check_authority()?;
    array
        .seal()
        .map_err(|denial| WorthQueryApplicationAttemptDenial::allocation_denied(subject, denial))
}
