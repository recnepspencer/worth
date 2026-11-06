//! Request-bound readmission of one authenticated symbol page.
use crate::durability::data::RelationalRecoveryStop;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use crate::symbols::data::SymbolTableSnapshot;

pub(crate) const SYMBOL_PAGE_CAPACITY: usize = 256;

pub(crate) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
    locator: RelationalDurableMemberLocator,
    subject: RelationalStorageSubject,
) -> Result<SymbolTableSnapshot, Failure> {
    let failure =
        |cause| Failure::StorageUnavailable(RelationalStorageUnavailable { subject, cause });
    let payload = member_frame::decode(bytes, locator, MemberKind::Symbols).map_err(failure)?;
    SymbolTableSnapshot::readmit_current(execution, payload)
        .map_err(|stop| readmission_failure(stop, subject))
}

fn readmission_failure(stop: RelationalRecoveryStop, subject: RelationalStorageSubject) -> Failure {
    let failure =
        |cause| Failure::StorageUnavailable(RelationalStorageUnavailable { subject, cause });
    match stop {
        RelationalRecoveryStop::Request(cause) => Failure::Request(cause),
        RelationalRecoveryStop::Allocation(_) => failure(Cause::MemoryExhausted),
        RelationalRecoveryStop::Content(cause) => Failure::Content(cause),
        RelationalRecoveryStop::Durability(_) | RelationalRecoveryStop::AdmissionImage(_) => {
            failure(Cause::Damaged)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_size_refusal_survives_symbol_page_readmission() {
        let refusal =
            RelationalRecoveryStop::Content(crate::content::RelationalContentDenial::SizeLimit {
                observed: 97,
                maximum: 31,
            });
        let subject = RelationalStorageSubject::Symbols {
            first: crate::symbols::data::Symbol(1),
            last: crate::symbols::data::Symbol(2),
        };
        assert!(matches!(
            readmission_failure(refusal, subject),
            Failure::Content(crate::content::RelationalContentDenial::SizeLimit {
                observed: 97,
                maximum: 31,
            })
        ));
    }
}
