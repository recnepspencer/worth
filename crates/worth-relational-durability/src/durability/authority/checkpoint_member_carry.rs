//! Destination membership governs carry eligibility, independently of shared
//! cells or the source runtime's last selected manifest. Foreign metadata may
//! be encoded again only when its complete contents are already resident.
use super::checkpoint_capture::CapturedCheckpointBasis;
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec::member_frame;
use contracts::lineage::data::graph::LineageNodeFieldsPort as _;
use std::collections::BTreeSet;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;

pub(super) fn may_carry_metadata(
    basis: &CapturedCheckpointBasis,
    prior: Option<&RelationalCheckpointManifest>,
    members: Option<&BTreeSet<RelationalDurableMemberLocator>>,
) -> Result<bool, DurabilityError> {
    let (Some(prior), Some(members)) = (prior, members) else {
        return Ok(true);
    };
    if prior
        .symbols
        .iter()
        .chain(&prior.lineage)
        .all(|page| members.contains(&page.member))
        && prior
            .symbol_buckets
            .iter()
            .flat_map(|bucket| &bucket.pages)
            .all(|page| members.contains(page))
        && prior
            .lineage_events
            .iter()
            .all(|page| members.contains(&page.member))
    {
        return Ok(true);
    }
    if !basis.lineage_event_pages_resident {
        let missing = prior
            .lineage_events
            .iter()
            .find(|page| !members.contains(&page.member));
        if let Some(page) = missing {
            return Err(unavailable(
                contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                    first: contracts::identity::data::primitives::LineageId(page.first_node),
                    last: contracts::identity::data::primitives::LineageId(page.last_node),
                },
            ));
        }
    }
    if !basis.symbol_pages_resident {
        return Err(unavailable(
            contracts::storage::data::storage_unavailable::RelationalStorageSubject::Symbols {
                first: contracts::symbols::data::Symbol(0),
                last: contracts::symbols::data::Symbol(u32::MAX - 1),
            },
        ));
    }
    for page in &prior.lineage {
        let nodes = basis
            .lineage_nodes
            .iter()
            .filter(|node| page.contains(node.owner_lineage_id().0))
            .collect::<Vec<_>>();
        let encoded = member_frame::encode(MemberKind::Lineage, &nodes).map_err(|_| {
            unavailable(
                contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                    first: contracts::identity::data::primitives::LineageId(page.first),
                    last: contracts::identity::data::primitives::LineageId(page.last),
                },
            )
        })?;
        if encoded.0 != page.member {
            return Err(unavailable(
                contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                    first: contracts::identity::data::primitives::LineageId(page.first),
                    last: contracts::identity::data::primitives::LineageId(page.last),
                },
            ));
        }
    }
    Ok(false)
}

pub(super) fn unavailable(
    subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject,
) -> DurabilityError {
    contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
        subject,
        cause: contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::SourceUnavailable,
    }
    .into()
}
