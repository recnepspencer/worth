//! Admission converts real compiler operations into checked Execution work.
use super::super::admission::CompilationWork;
use super::*;
use worth_execution::{MapKernelContext, MapKernelStop};

pub(super) fn admit(
    context: &mut MapKernelContext<'_, '_>,
    work: CompilationWork<'_>,
) -> Result<(), MapKernelFailure<DiagnosticContractDenial>> {
    let overflow = || MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow);
    let units = match work {
        CompilationWork::Visit => 1,
        CompilationWork::Copy(bytes) => u64::try_from(bytes).map_err(|_| overflow())?.max(1),
        CompilationWork::Scan { entries, parts } | CompilationWork::Lookup { entries, parts } => {
            // A comparison can examine at most each target text plus its end.
            // entries+1 is a portable upper bound, without assuming a particular
            // standard-library tree node width or height implementation.
            let width = parts
                .iter()
                .try_fold(1u64, |sum, text| {
                    sum.checked_add(u64::try_from(text.len()).ok()?.checked_add(1)?)
                })
                .ok_or_else(overflow)?;
            u64::try_from(entries)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(width))
                .ok_or_else(overflow)?
        }
    };
    checkpoint(context, units).map_err(MapKernelFailure::Stop)
}
