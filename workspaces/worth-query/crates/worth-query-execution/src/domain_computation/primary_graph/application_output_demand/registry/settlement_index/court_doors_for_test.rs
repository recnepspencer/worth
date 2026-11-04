//! Sealed test doors over this module's private members.
use super::*;

impl SettlementIndex {
    #[doc(hidden)]
    pub fn fill_prepared_for_test(
        posting: &Posting,
        expected: &RecordedSettlementIdentity,
        identity: Arc<RecordedSettlementIdentity>,
        key: Arc<WorthQueryOutputDemandKey>,
    ) {
        Self::fill_prepared(posting, expected, identity, key)
    }
}
