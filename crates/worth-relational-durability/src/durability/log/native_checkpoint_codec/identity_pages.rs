//! `CHK24` capture of the record-identity tables as paged directory members.
use std::sync::Arc;

use super::manifest_capture::corrupt;
use crate::durability::data::DurableRecordIdentityState;
use contracts::durability::data::recovery_errors::DurabilityError;
use worth_execution::ExecutionImmutableBytes;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::PagedIdentityBase;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;
use worth_relational_storage::internal_port::storage::data::record_identity_pages::{
    encode_record_identity, IdentityDirectories, PagedRecordIdentity, PagedRecordIdentityRoots,
    IDENTITY_PAGE_BYTES,
};
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::{
    CollectedMembers, ColdMemberSource,
};
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

/// Encodes the captured allocator image as four paged directories, returning
/// the manifest's roots and page roster and the page members to publish.
pub(crate) fn encode_identity_pages(
    state: &DurableRecordIdentityState,
) -> Result<
    (
        PagedRecordIdentity,
        Vec<(RelationalDurableMemberLocator, ExecutionImmutableBytes)>,
    ),
    DurabilityError,
> {
    let mut sink = CollectedMembers::default();
    let roots = encode_record_identity(state, &mut sink)
        .map_err(|cause| corrupt(format!("record identity directory: {cause:?}")))?;
    let members = sink
        .members
        .into_iter()
        .map(|(locator, bytes)| {
            (
                locator,
                ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice())),
            )
        })
        .collect();
    Ok((roots, members))
}

fn pages(source: &Arc<dyn RelationalDurableColdSource>) -> ColdMemberSource {
    ColdMemberSource::new(source.clone(), IDENTITY_PAGE_BYTES as u64)
}

/// An accountant for page faults when the runtime declared no budget.
fn unbudgeted() -> Arc<WorkingSetAccountant> {
    WorkingSetAccountant::new(contracts::config::data::residency_budget::ResidencyBudget {
        head_working_set_bytes: u64::MAX,
        historical_working_set_bytes: u64::MAX,
        max_in_flight_fill_bytes: u64::MAX,
        streaming_window_bytes: u32::MAX,
    })
}

/// The whole allocator image. Expanded decode and inspection only; reopen
/// uses [`open_paged_identity`].
pub(crate) fn load_identity(
    roots: &PagedRecordIdentityRoots,
    source: &Arc<dyn RelationalDurableColdSource>,
) -> Result<DurableRecordIdentityState, DurabilityError> {
    let directories = IdentityDirectories::open(roots, unbudgeted());
    roots
        .load_resident(&directories, &pages(source))
        .map_err(|cause| corrupt(format!("record identity directory: {cause:?}")))
}

/// The reopened allocator: only `roots` are retained; partitions fault their
/// rows through `accountant` on first use.
pub(crate) fn open_paged_identity(
    paged: PagedRecordIdentity,
    source: &Arc<dyn RelationalDurableColdSource>,
    accountant: Option<Arc<WorkingSetAccountant>>,
) -> RecordIdentitySubsystem {
    RecordIdentitySubsystem::from_paged(PagedIdentityBase::open(
        paged,
        accountant.unwrap_or_else(unbudgeted),
        Arc::new(pages(source)),
    ))
}

/// Installs an unchanged reopened allocator's paged directories into a native
/// capture, carrying members already present in the destination and copying
/// verified bytes otherwise without materializing resident rows.
pub(crate) fn apply_carried_identity(
    captured: &mut super::CapturedManifest,
    base: &PagedIdentityBase,
    prior: Option<&crate::durability::data::RelationalCheckpointManifest>,
    carry_metadata: bool,
    current_members: Option<&std::collections::BTreeSet<RelationalDurableMemberLocator>>,
) -> Result<(), DurabilityError> {
    let paged = base.paged();
    for locator in &captured.manifest.record_identity.pages {
        captured.members.remove(locator);
    }
    captured.manifest.record_identity = paged.clone();
    for &locator in &paged.pages {
        let may_carry = current_members.map_or(
            carry_metadata
                && prior.is_some_and(|prior| {
                    prior.record_identity.pages.binary_search(&locator).is_ok()
                }),
            |held| held.contains(&locator),
        );
        let member = if may_carry {
            crate::durability::data::RelationalCheckpointMember::Carried { locator }
        } else {
            let bytes = base.page_bytes(locator).map_err(|cause| {
                DurabilityError::new(
                    contracts::durability::data::recovery_errors::RecoveryFailureClass::ColdStorageUnavailable,
                    format!("record identity page unavailable: {cause:?}"),
                )
            })?;
            crate::durability::data::RelationalCheckpointMember::New {
                locator,
                bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(
                    bytes.into_boxed_slice(),
                )),
            }
        };
        if matches!(
            member,
            crate::durability::data::RelationalCheckpointMember::New { .. }
        ) {
            captured.members.insert(locator, member);
        } else {
            captured.members.entry(locator).or_insert(member);
        }
    }
    Ok(())
}
