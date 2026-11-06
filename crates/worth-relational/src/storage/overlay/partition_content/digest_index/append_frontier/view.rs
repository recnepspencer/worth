//! Borrowed original right spine with the unchanged siblings/last wire grammar.
use super::*;
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

pub(in crate::storage::overlay::partition_content) struct FrontierView<'a> {
    root: Option<&'a Node>,
    siblings: usize,
    last: Option<(u128, worth_foundational::CanonicalDigestId)>,
}
impl DigestIndex {
    pub(in crate::storage::overlay::partition_content) fn append_frontier_view(
        &self,
        last_value: Option<[u8; 32]>,
    ) -> Option<FrontierView<'_>> {
        let mut siblings = 0usize;
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            if let Some(children) = &node.children {
                siblings = siblings.checked_add(1)?;
                current = Some(&children[1]);
            } else {
                if node.bit != 128 {
                    return None;
                }
                let value = last_value.or_else(|| {
                    self.history_last_value
                        .as_ref()
                        .filter(|last| last.key == node.key)
                        .map(|last| last.value)
                })?;
                if hash(1, &[&node.key.to_be_bytes(), &value]) != node.digest {
                    return None;
                }
                return Some(FrontierView {
                    root: self.root.as_deref(),
                    siblings,
                    last: Some((node.key, worth_foundational::CanonicalDigestId::new(value))),
                });
            }
        }
        last_value.is_none().then_some(FrontierView {
            root: None,
            siblings,
            last: None,
        })
    }
}
impl Serialize for FrontierView<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("HistoryAppendFrontier", 2)?;
        state.serialize_field("siblings", &Siblings(self))?;
        state.serialize_field("last", &self.last)?;
        state.end()
    }
}
struct Siblings<'a>(&'a FrontierView<'a>);
impl Serialize for Siblings<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.siblings))?;
        let mut current = self.0.root;
        while let Some(node) = current {
            let Some(children) = &node.children else {
                break;
            };
            sequence.serialize_element(&HistoryFrontierSibling {
                key: children[0].key,
                bit: children[0].bit,
                digest: worth_foundational::CanonicalDigestId::new(children[0].digest),
                parent_bit: node.bit,
            })?;
            current = Some(&children[1]);
        }
        sequence.end()
    }
}
