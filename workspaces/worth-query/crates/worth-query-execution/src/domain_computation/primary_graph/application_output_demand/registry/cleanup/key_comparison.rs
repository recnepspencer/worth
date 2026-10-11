//! The selected cleanup key owns its comparison width.
use super::{work_denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandKey};

pub(super) fn key_work(
    key: &WorthQueryOutputDemandKey,
) -> Result<u64, WorthQueryOutputDemandDenial> {
    super::super::key_comparison::comparison_work([key])
        .and_then(|bytes| u64::try_from(bytes.maximum).ok())
        .ok_or_else(work_denial)
}

#[cfg(test)]
#[path = "key_comparison/tests.rs"]
mod tests;
