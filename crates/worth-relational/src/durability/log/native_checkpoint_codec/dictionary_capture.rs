//! Bounded dictionary member encoding for identity reads and spelling lookup.
use super::manifest_capture::{add, corrupt};
use super::member_frame::{self, MemberKind};
use crate::durability::checkpoints::symbol_pages::SYMBOL_PAGE_CAPACITY;
use crate::durability::data::{
    RelationalCheckpointMember, RelationalCheckpointPage, RelationalCheckpointSymbolBucket,
    RelationalNativeCheckpointCaptureDenial,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::symbols::data::SymbolTableSnapshot;
use std::collections::BTreeMap;

pub(super) struct CapturedDictionary {
    pub(super) by_id: Vec<RelationalCheckpointPage>,
    pub(super) by_text: Vec<RelationalCheckpointSymbolBucket>,
}
pub(super) fn capture(
    snapshot: &SymbolTableSnapshot,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    members: &mut BTreeMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<CapturedDictionary, RelationalNativeCheckpointCaptureDenial> {
    let mut entries = snapshot.entries.clone();
    entries.sort_by_key(|(symbol, _)| *symbol);
    let mut symbol_pages = Vec::new();
    for page in entries.chunk_by(|a, b| {
        RelationalCheckpointPage::range_of(u64::from(a.0 .0))
            == RelationalCheckpointPage::range_of(u64::from(b.0 .0))
    }) {
        policy.check_live()?;
        let value = crate::symbols::data::SymbolTableSnapshot {
            entries: page.to_vec(),
        };
        let member = add(
            members,
            member_frame::encode(MemberKind::Symbols, &value)
                .map_err(|cause| corrupt(format!("symbol member: {cause:?}")))?,
        );
        symbol_pages.push(
            RelationalCheckpointPage::from_ids(page.iter().map(|(id, _)| u64::from(id.0)), member)
                .ok_or_else(|| corrupt("invalid captured symbol page membership"))?,
        );
    }
    let mut buckets = BTreeMap::<u8, Vec<_>>::new();
    for (symbol, text) in &snapshot.entries {
        use sha2::{Digest, Sha256};
        let bucket = Sha256::digest(text.as_str().as_bytes())[0];
        buckets
            .entry(bucket)
            .or_default()
            .push((*symbol, text.clone()));
    }
    let mut symbol_buckets = Vec::new();
    for (bucket, mut entries) in buckets {
        entries.sort_by_key(|(symbol, _)| *symbol);
        let mut pages = Vec::new();
        for entries in entries.chunks(SYMBOL_PAGE_CAPACITY) {
            policy.check_live()?;
            let page = crate::symbols::data::SymbolTableSnapshot {
                entries: entries.to_vec(),
            };
            pages.push(add(
                members,
                member_frame::encode(MemberKind::Symbols, &page)
                    .map_err(|cause| corrupt(format!("symbol bucket member: {cause:?}")))?,
            ));
        }
        symbol_buckets.push(RelationalCheckpointSymbolBucket { bucket, pages });
    }
    Ok(CapturedDictionary {
        by_id: symbol_pages,
        by_text: symbol_buckets,
    })
}
