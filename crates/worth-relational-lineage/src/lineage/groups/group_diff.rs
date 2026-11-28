//! Bounded structural-sharing diffing, counting, and cold-base rebasing for
//! [`BranchGroupState`] and [`SchemeGroups`].

use std::collections::BTreeMap;

use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::builder::PageChange;
use crate::lineage::naming_pages::codec::NamingCodec;
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::tables::{self, scoped_prefix};
use crate::lineage::naming_pages::BranchNamingRoot;
use crate::lineage::tiered::{NamingBaseLink, TieredMap};

use super::branch_state::{BranchGroupState, SchemeGroups};

fn rebase_table<K, V>(
    target: &mut TieredMap<K, V>,
    rebuilt: &TieredMap<K, V>,
    source: &TieredMap<K, V>,
    prefix: &[u8],
) -> Result<(), Cause>
where
    K: NamingCodec + Ord + Clone,
    V: NamingCodec + Clone + PartialEq,
{
    let mut desired: BTreeMap<K, V> = BTreeMap::new();
    rebuilt.scan_merged(prefix, |k, v| {
        desired.insert(k, v.into_owned());
        Ok(true)
    })?;
    let mut existing: BTreeMap<K, V> = BTreeMap::new();
    source.scan_merged(prefix, |k, v| {
        existing.insert(k, v.into_owned());
        Ok(true)
    })?;
    for (k, v) in &desired {
        if existing.get(k) != Some(v) {
            target.insert(k.clone(), v.clone());
        }
    }
    for k in existing.keys() {
        if !desired.contains_key(k) {
            target.remove_blind(k);
        }
    }
    Ok(())
}

impl SchemeGroups {
    pub(crate) fn delta_len(&self) -> usize {
        self.live.delta_len()
            + self.members.delta_len()
            + self.retired.delta_len()
            + self.member_owner.delta_len()
    }

    fn count_diff_against_bounded(
        &self,
        base: &Self,
        has_cold_base: bool,
        budget: usize,
        visited: &mut usize,
    ) -> Option<usize> {
        let mut total = 0usize;
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(total);
                total += self.$field.count_diff_against_bounded(
                    &base.$field,
                    has_cold_base,
                    rem,
                    visited,
                )?;
                if total > budget {
                    return None;
                }
            };
        }
        step!(live);
        step!(members);
        step!(retired);
        step!(member_owner);
        Some(total)
    }

    fn diff_encoded_against_bounded(
        &self,
        base: &Self,
        cold_base: Option<(&ColdNamingStore, &BranchNamingRoot)>,
        budget: usize,
    ) -> Result<Option<Vec<(Vec<u8>, PageChange)>>, Cause> {
        let mut out = Vec::new();
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(out.len());
                let Some(part) =
                    self.$field
                        .diff_encoded_against_bounded(&base.$field, cold_base, rem)?
                else {
                    return Ok(None);
                };
                out.extend(part);
            };
        }
        step!(live);
        step!(members);
        step!(retired);
        step!(member_owner);
        Ok(Some(out))
    }

    fn encoded_puts_bounded(&self, budget: usize) -> Option<Vec<(Vec<u8>, PageChange)>> {
        let mut out = Vec::new();
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(out.len());
                out.extend(self.$field.encoded_puts_bounded(rem)?);
            };
        }
        step!(live);
        step!(members);
        step!(retired);
        step!(member_owner);
        Some(out)
    }

    fn encoded_tombstones_bounded(&self, budget: usize) -> Option<Vec<(Vec<u8>, PageChange)>> {
        let mut out = Vec::new();
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(out.len());
                out.extend(self.$field.encoded_tombstones_bounded(rem)?);
            };
        }
        step!(live);
        step!(members);
        step!(retired);
        step!(member_owner);
        Some(out)
    }
}

impl BranchGroupState {
    pub(crate) fn delta_len(&self) -> usize {
        self.schemes.values().map(SchemeGroups::delta_len).sum()
    }

    pub(crate) fn count_diff_against_bounded(
        &self,
        base: &Self,
        has_cold_base: bool,
        budget: usize,
        visited: &mut usize,
    ) -> Option<usize> {
        let mut total = 0usize;
        for (id, scheme) in &self.schemes {
            let rem = budget.saturating_sub(total);
            match base.schemes.get(id) {
                Some(base_scheme) => {
                    let eff_cold = match (scheme.base().is_some(), base_scheme.base().is_some()) {
                        (true, true) => has_cold_base,
                        (false, false) => false,
                        _ => return None,
                    };
                    total +=
                        scheme.count_diff_against_bounded(base_scheme, eff_cold, rem, visited)?;
                }
                None => {
                    if scheme.base().is_some() || scheme.delta_len() > rem {
                        return None;
                    }
                    total += scheme.delta_len();
                }
            }
            if total > budget {
                return None;
            }
        }
        for (id, base_scheme) in &base.schemes {
            if self.schemes.contains_key(id) {
                continue;
            }
            if base_scheme.base().is_some()
                || base_scheme.delta_len() > budget.saturating_sub(total)
            {
                return None;
            }
            total += base_scheme.delta_len();
        }
        Some(total)
    }

    pub(crate) fn diff_encoded_against_bounded(
        &self,
        base: &Self,
        cold_base: Option<(&ColdNamingStore, &BranchNamingRoot)>,
        budget: usize,
    ) -> Result<Option<Vec<(Vec<u8>, PageChange)>>, Cause> {
        let mut out = Vec::new();
        for (id, scheme) in &self.schemes {
            let rem = budget.saturating_sub(out.len());
            match base.schemes.get(id) {
                Some(base_scheme) => {
                    let eff_cold = match (scheme.base().is_some(), base_scheme.base().is_some()) {
                        (true, true) => cold_base,
                        (false, false) => None,
                        _ => return Ok(None),
                    };
                    let Some(part) =
                        scheme.diff_encoded_against_bounded(base_scheme, eff_cold, rem)?
                    else {
                        return Ok(None);
                    };
                    out.extend(part);
                }
                None => {
                    if scheme.base().is_some() {
                        return Ok(None);
                    }
                    let Some(part) = scheme.encoded_puts_bounded(rem) else {
                        return Ok(None);
                    };
                    out.extend(part);
                }
            }
        }
        for (id, base_scheme) in &base.schemes {
            if self.schemes.contains_key(id) {
                continue;
            }
            if base_scheme.base().is_some() {
                return Ok(None);
            }
            let rem = budget.saturating_sub(out.len());
            let Some(part) = base_scheme.encoded_tombstones_bounded(rem) else {
                return Ok(None);
            };
            out.extend(part);
        }
        Ok(Some(out))
    }

    pub(crate) fn encoded_puts_bounded(&self, budget: usize) -> Option<Vec<(Vec<u8>, PageChange)>> {
        let mut out = Vec::new();
        for scheme in self.schemes.values() {
            if scheme.base().is_some() {
                return None;
            }
            let rem = budget.saturating_sub(out.len());
            out.extend(scheme.encoded_puts_bounded(rem)?);
        }
        Some(out)
    }

    /// Rebases a historically rebuilt resident group state onto `source`'s
    /// shared `CowMap`s and cold base (if any), preserving structural sharing
    /// for surviving schemes even when post-cutoff resident schemes were added
    /// to `source`.
    pub(crate) fn rebase_over(
        self,
        source: &Self,
        base: Option<NamingBaseLink>,
    ) -> Result<Self, Cause> {
        let compatible = self
            .schemes
            .iter()
            .all(|(id, s)| source.schemes.get(id).is_some_and(|b| b.rule == s.rule))
            && (base.is_none()
                || source
                    .schemes
                    .iter()
                    .all(|(id, b)| self.schemes.contains_key(id) || b.base().is_none()));
        if !compatible {
            return Ok(self);
        }
        let mut out = source.fork_subset(self.schemes.keys().map(String::as_str), base);
        out.earliest_commit = self.earliest_commit;
        out.latest_commit = self.latest_commit;
        for (id, rebuilt) in &self.schemes {
            let src = &source.schemes[id];
            let tgt = out.schemes.get_mut(id).expect("matching scheme");
            rebase_table(
                &mut tgt.live,
                &rebuilt.live,
                &src.live,
                &scoped_prefix(tables::GROUP_LIVE, id),
            )?;
            rebase_table(
                &mut tgt.members,
                &rebuilt.members,
                &src.members,
                &scoped_prefix(tables::GROUP_MEMBERS, id),
            )?;
            rebase_table(
                &mut tgt.retired,
                &rebuilt.retired,
                &src.retired,
                &scoped_prefix(tables::GROUP_RETIRED, id),
            )?;
            rebase_table(
                &mut tgt.member_owner,
                &rebuilt.member_owner,
                &src.member_owner,
                &scoped_prefix(tables::GROUP_MEMBER_OWNER, id),
            )?;
        }
        Ok(out)
    }
}
