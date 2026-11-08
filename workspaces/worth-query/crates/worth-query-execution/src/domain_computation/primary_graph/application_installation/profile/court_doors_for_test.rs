//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationProfile {
    #[doc(hidden)]
    pub fn publication_override_for_test(
        self,
        maximum_records: Option<std::num::NonZeroUsize>,
    ) -> Option<worth_relational::facade::config::PublicationConfig> {
        Self::publication_override(self, maximum_records)
    }
}

impl WorthQueryApplicationProfile {
    #[doc(hidden)]
    pub fn relational_profile_for_test(
        self,
    ) -> worth_relational::facade::config::RelationalRuntimeProfile {
        Self::relational_profile(self)
    }
}
