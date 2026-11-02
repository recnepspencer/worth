//! Authored incarnation graph; the oracle reads no index, descriptors or native rows.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
type Key = (String, u64);
#[derive(Clone)]
struct Item {
    incarnation: u64,
    name: String,
    enabled: bool,
}
#[derive(Clone, Default)]
pub(super) struct Model {
    items: BTreeMap<String, Item>,
    roots: BTreeMap<String, BTreeSet<Key>>,
    edges: BTreeSet<(Key, Key)>,
    scope_ids: BTreeMap<String, (u32, u64, u32)>,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Value {
    pub identity: String,
    pub name: String,
    pub children: Vec<Value>,
    pub parents: Vec<Value>,
}
impl Model {
    pub fn record_scope(&mut self, label: &str, actual: (u32, u64, u32)) {
        assert!(self.scope_ids.insert(label.to_owned(), actual).is_none());
    }
    pub fn scope(&self, label: &str) -> (u32, u64, u32) {
        self.scope_ids[label]
    }
    pub fn directory(&self, actual: (u32, u64, u32)) -> &str {
        self.scope_ids
            .iter()
            .find(|(_, id)| **id == actual)
            .expect("the callback uses an actually authored Directory")
            .0
    }
    fn key(&self, name: &str) -> Key {
        (name.to_owned(), self.items[name].incarnation)
    }
    pub fn apply(&mut self, edit: &Edit) {
        match edit {
            Edit::Directory { identity } => {
                self.roots.entry(identity.clone()).or_default();
            }
            Edit::Item {
                directory,
                identity,
                name,
                root,
                incarnation,
            } => {
                assert!(self
                    .items
                    .insert(
                        identity.clone(),
                        Item {
                            incarnation: *incarnation,
                            name: name.clone(),
                            enabled: true
                        }
                    )
                    .is_none());
                if *root {
                    let key = self.key(identity);
                    self.roots.entry(directory.clone()).or_default().insert(key);
                }
            }
            Edit::Name { item, value } => self.items.get_mut(item).unwrap().name = value.clone(),
            Edit::Enabled { item, value } => self.items.get_mut(item).unwrap().enabled = *value,
            Edit::Link { from, to, .. } => {
                assert!(self.edges.insert((self.key(from), self.key(to))));
            }
            Edit::Unlink { from, to } => {
                assert!(self.edges.remove(&(self.key(from), self.key(to))));
            }
            Edit::Share { directory, item } => {
                let key = self.key(item);
                self.roots.entry(directory.clone()).or_default().insert(key);
            }
            Edit::Retire { item } => {
                assert!(self.items.remove(item).is_some());
            }
        }
    }
    fn value(&self, key: &Key, levels: usize) -> Option<Value> {
        let item = self
            .items
            .get(&key.0)
            .filter(|item| item.incarnation == key.1)?;
        let mut children = if levels == 0 {
            Vec::new()
        } else {
            self.edges
                .iter()
                .filter(|(from, _)| from == key)
                .filter_map(|(_, to)| self.value(to, levels - 1))
                .collect()
        };
        children.sort();
        Some(Value {
            identity: key.0.clone(),
            name: item.name.clone(),
            children,
            parents: Vec::new(),
        })
    }
    pub fn expected(&self, scope: &str, prefix: &str, enabled_only: bool) -> Vec<Value> {
        let mut rows: Vec<_> = self
            .roots
            .get(scope)
            .into_iter()
            .flatten()
            .filter(|key| {
                self.items.get(&key.0).is_some_and(|item| {
                    item.incarnation == key.1
                        && (!enabled_only || item.enabled)
                        && item.name.starts_with(prefix)
                })
            })
            .filter_map(|key| {
                let mut value = self.value(key, 3)?;
                value.parents = self
                    .edges
                    .iter()
                    .filter(|(_, to)| to == key)
                    .filter_map(|(from, _)| self.value(from, 1))
                    .collect();
                value.parents.sort();
                Some(value)
            })
            .collect();
        rows.sort();
        rows
    }
}
pub(super) fn values(rows: &[query::ItemRow]) -> Vec<Value> {
    let mut rows: Vec<_> = rows
        .iter()
        .map(|row| Value {
            identity: row.identity.clone(),
            name: row.name.clone(),
            children: children(&row.children),
            parents: children(&row.parents),
        })
        .collect();
    rows.sort();
    rows
}
fn children(nodes: &[super::super::shape::Node]) -> Vec<Value> {
    let mut rows: Vec<_> = nodes
        .iter()
        .map(|row| Value {
            identity: row.identity.clone(),
            name: row.name.clone(),
            children: children(&row.children),
            parents: Vec::new(),
        })
        .collect();
    rows.sort();
    rows
}
pub(super) fn exact(rows: &[query::ItemRow]) -> Vec<query::ItemRow> {
    let mut rows = rows.to_vec();
    rows.sort_by(|a, b| a.identity.cmp(&b.identity));
    rows
}
