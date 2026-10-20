use std::num::NonZeroUsize;

use worth_query_admission::facade::graph_read_access::WorthQueryGraphReadBudget;

const DEFAULT_CONCURRENT_GRAPH_WORK: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryResourceProfileDenial {
    ZeroInlineIndexBytes,
    ZeroResultBytesPerRoot,
    ZeroIntermediateSetSize,
    ZeroConcurrentGraphWork,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryResourceProfile {
    maximum_work: Option<NonZeroUsize>,
    maximum_inline_index_bytes: Option<NonZeroUsize>,
    maximum_result_bytes_per_root: Option<NonZeroUsize>,
    maximum_intermediate_set_size: Option<NonZeroUsize>,
    maximum_concurrent_graph_work: NonZeroUsize,
}

impl WorthQueryApplicationQueryResourceProfile {
    pub fn bounded(
        maximum_inline_index_bytes: usize,
        maximum_result_bytes_per_root: usize,
        maximum_intermediate_set_size: usize,
        maximum_concurrent_graph_work: usize,
    ) -> Result<Self, WorthQueryApplicationQueryResourceProfileDenial> {
        Ok(Self {
            maximum_work: None,
            maximum_inline_index_bytes: Some(
                NonZeroUsize::new(maximum_inline_index_bytes)
                    .ok_or(WorthQueryApplicationQueryResourceProfileDenial::ZeroInlineIndexBytes)?,
            ),
            maximum_result_bytes_per_root: Some(
                NonZeroUsize::new(maximum_result_bytes_per_root).ok_or(
                    WorthQueryApplicationQueryResourceProfileDenial::ZeroResultBytesPerRoot,
                )?,
            ),
            maximum_intermediate_set_size: Some(
                NonZeroUsize::new(maximum_intermediate_set_size).ok_or(
                    WorthQueryApplicationQueryResourceProfileDenial::ZeroIntermediateSetSize,
                )?,
            ),
            maximum_concurrent_graph_work: NonZeroUsize::new(maximum_concurrent_graph_work)
                .ok_or(WorthQueryApplicationQueryResourceProfileDenial::ZeroConcurrentGraphWork)?,
        })
    }

    pub const fn with_maximum_work(mut self, maximum_work: NonZeroUsize) -> Self {
        self.maximum_work = Some(maximum_work);
        self
    }

    /// Select simultaneous graph-work admission independently of optional query quotas.
    pub const fn with_maximum_concurrent_graph_work(mut self, maximum: NonZeroUsize) -> Self {
        self.maximum_concurrent_graph_work = maximum;
        self
    }

    pub const fn maximum_work(self) -> Option<NonZeroUsize> {
        self.maximum_work
    }

    pub const fn maximum_inline_index_bytes(self) -> Option<NonZeroUsize> {
        self.maximum_inline_index_bytes
    }

    pub const fn maximum_result_bytes_per_root(self) -> Option<NonZeroUsize> {
        self.maximum_result_bytes_per_root
    }

    pub const fn maximum_intermediate_set_size(self) -> Option<NonZeroUsize> {
        self.maximum_intermediate_set_size
    }

    pub const fn maximum_concurrent_graph_work(self) -> NonZeroUsize {
        self.maximum_concurrent_graph_work
    }

    pub(crate) fn admission_budget(
        self,
        maximum_result_count: NonZeroUsize,
        request_maximum_work: Option<NonZeroUsize>,
    ) -> WorthQueryGraphReadBudget {
        let (index, result, intermediate) =
            self.admission_limits(maximum_result_count, request_maximum_work);
        WorthQueryGraphReadBudget::policy(index, result, intermediate)
    }

    /// A request may narrow the host's whole-result envelope before canonical
    /// planning. This preserves every other host admission dimension.
    pub(crate) fn admission_budget_with_result_cap(
        self,
        maximum_result_count: NonZeroUsize,
        request_maximum_work: Option<NonZeroUsize>,
        maximum_result_bytes: Option<NonZeroUsize>,
    ) -> WorthQueryGraphReadBudget {
        let Some(cap) = maximum_result_bytes else {
            return self.admission_budget(maximum_result_count, request_maximum_work);
        };
        let (index, result, intermediate) =
            self.admission_limits(maximum_result_count, request_maximum_work);
        WorthQueryGraphReadBudget::policy(
            index,
            result.into_iter().chain(Some(cap.get())).min(),
            intermediate,
        )
    }

    pub(crate) fn admission_budget_admitted<Stop>(
        self,
        maximum_result_count: NonZeroUsize,
        request_maximum_work: Option<NonZeroUsize>,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        WorthQueryGraphReadBudget,
        worth_query_admission::integration::WorthQueryCanonicalIdentityStop<Stop>,
    > {
        admit(4, 0).map_err(
            worth_query_admission::integration::WorthQueryCanonicalIdentityStop::Admission,
        )?;
        let (index, result, intermediate) =
            self.admission_limits(maximum_result_count, request_maximum_work);
        WorthQueryGraphReadBudget::policy_admitted(index, result, intermediate, admit)
    }

    fn admission_limits(
        self,
        maximum_result_count: NonZeroUsize,
        request_maximum_work: Option<NonZeroUsize>,
    ) -> (Option<usize>, Option<usize>, Option<usize>) {
        (
            self.maximum_inline_index_bytes.map(NonZeroUsize::get),
            self.maximum_result_bytes_per_root
                .map(|bytes| bytes.get().saturating_mul(maximum_result_count.get())),
            self.maximum_intermediate_set_size
                .into_iter()
                .chain(request_maximum_work)
                .map(NonZeroUsize::get)
                .min(),
        )
    }
}

impl Default for WorthQueryApplicationQueryResourceProfile {
    fn default() -> Self {
        Self {
            maximum_work: None,
            maximum_inline_index_bytes: None,
            maximum_result_bytes_per_root: None,
            maximum_intermediate_set_size: None,
            maximum_concurrent_graph_work: NonZeroUsize::new(DEFAULT_CONCURRENT_GRAPH_WORK)
                .expect("default concurrent graph work is non-zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryApplicationQueryResourceProfile;
    use std::num::NonZeroUsize;

    #[test]
    fn request_limits_only_narrow_the_installed_profile() {
        let profile =
            WorthQueryApplicationQueryResourceProfile::bounded(10_000, 2_000, 100, 7).unwrap();
        assert_eq!(profile.maximum_concurrent_graph_work().get(), 7);
        let broad_request = profile.admission_budget(
            NonZeroUsize::new(3).unwrap(),
            Some(NonZeroUsize::new(500).unwrap()),
        );
        assert_eq!(broad_request.max_inline_index_bytes(), Some(10_000));
        assert_eq!(broad_request.max_inline_result_bytes(), Some(6_000));
        assert_eq!(broad_request.max_inline_intermediate_set_size(), Some(100));

        let narrow_request = profile.admission_budget(
            NonZeroUsize::new(1).unwrap(),
            Some(NonZeroUsize::new(40).unwrap()),
        );
        assert_eq!(narrow_request.max_inline_index_bytes(), Some(10_000));
        assert_eq!(narrow_request.max_inline_result_bytes(), Some(2_000));
        assert_eq!(narrow_request.max_inline_intermediate_set_size(), Some(40));
    }

    #[test]
    fn operational_work_is_optional_host_policy_not_a_result_formula() {
        let ordinary = WorthQueryApplicationQueryResourceProfile::default();
        assert_eq!(ordinary.maximum_work(), None);
        let configured = ordinary.with_maximum_work(NonZeroUsize::new(700).unwrap());
        assert_eq!(configured.maximum_work(), NonZeroUsize::new(700));
        assert_eq!(
            configured.maximum_result_bytes_per_root(),
            ordinary.maximum_result_bytes_per_root()
        );
    }
}
