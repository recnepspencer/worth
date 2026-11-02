//! Actual root node/direction dispatch avoids searching unrelated anchor keys.
use super::{DefinitionScope, Key};
use crate::derived_relation::evaluation::{DefinitionDenial, Denial, Member, SourceRead};
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
type Endpoints = BTreeSet<Member>;
mod bucket;
use bucket::Bucket;
mod root_pages;
use root_pages::RootPages;
#[cfg(test)]
mod tests;

#[derive(Default)]
pub(in crate::derived_relation::evaluation) struct MemoValues {
    root: RootPages,
    named: BTreeMap<Key, Endpoints>,
}
fn charge(read: &mut impl SourceRead, width: usize, count: usize) -> Result<(), Denial> {
    let levels = (usize::BITS - count.max(1).leading_zeros()) as usize;
    let work = width
        .checked_mul(11)
        .and_then(|n| n.checked_mul(levels))
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    read.checkpoint(work)
}
impl MemoValues {
    pub(in crate::derived_relation::evaluation) fn get<'a>(
        &'a self,
        key: &Key,
        read: &mut impl SourceRead,
    ) -> Result<Option<&'a Endpoints>, Denial> {
        read.checkpoint(1)?;
        let result = match key.definition_scope {
            DefinitionScope::Root => self.root.get(key, read)?,
            DefinitionScope::Named => {
                charge(
                    read,
                    key.comparison_bytes()
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                    self.named.len(),
                )?;
                self.named.get(key)
            }
        };
        read.checkpoint(0)?;
        Ok(result)
    }
    pub(in crate::derived_relation::evaluation) fn remove(
        &mut self,
        key: &Key,
        read: &mut impl SourceRead,
    ) -> Result<Option<Endpoints>, Denial> {
        read.checkpoint(1)?;
        let result = match key.definition_scope {
            DefinitionScope::Root => self.root.remove(key, read)?,
            DefinitionScope::Named => {
                charge(
                    read,
                    key.comparison_bytes()
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                    self.named.len(),
                )?;
                self.named.remove(key)
            }
        };
        read.checkpoint(0)?;
        Ok(result)
    }
    pub(in crate::derived_relation::evaluation) fn insert(
        &mut self,
        key: Key,
        values: Endpoints,
        read: &mut impl SourceRead,
    ) -> Result<Option<Endpoints>, Denial> {
        read.checkpoint(1)?;
        let result = match key.definition_scope {
            DefinitionScope::Root => self.root.insert(key, values, read)?,
            DefinitionScope::Named => {
                charge(
                    read,
                    key.comparison_bytes()
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
                    self.named.len(),
                )?;
                read.retain(size_of::<(Key, Endpoints)>() + 4 * size_of::<usize>())?;
                self.named.insert(key, values)
            }
        };
        read.checkpoint(0)?;
        Ok(result)
    }
}
