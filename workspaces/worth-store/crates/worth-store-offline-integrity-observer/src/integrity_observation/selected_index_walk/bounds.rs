use super::super::families::index::OfflineBTreeNodeFacts;

/// A parent's separator interval is the authority for every descendant key.
#[derive(Clone, Default)]
pub(super) struct IndexKeyBounds {
    lower: Option<Vec<u8>>,
    upper: Option<Vec<u8>>,
}

impl IndexKeyBounds {
    pub(super) fn admits(&self, node: &OfflineBTreeNodeFacts) -> bool {
        node.keys.iter().all(|key| {
            self.lower.as_ref().is_none_or(|lower| lower <= key)
                && self.upper.as_ref().is_none_or(|upper| key < upper)
        })
    }

    pub(super) fn children(&self, node: &OfflineBTreeNodeFacts) -> Vec<Self> {
        (0..=node.keys.len())
            .map(|position| Self {
                lower: if position == 0 {
                    self.lower.clone()
                } else {
                    Some(node.keys[position - 1].clone())
                },
                upper: node
                    .keys
                    .get(position)
                    .cloned()
                    .or_else(|| self.upper.clone()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(keys: &[&[u8]]) -> OfflineBTreeNodeFacts {
        OfflineBTreeNodeFacts {
            level: 0,
            first_child: None,
            separator_children: Vec::new(),
            keys: keys.iter().map(|key| key.to_vec()).collect(),
            leaf_records: Vec::new(),
            previous_sibling: None,
            next_sibling: None,
            cell_count: keys.len() as u16,
        }
    }

    #[test]
    fn child_key_outside_parent_separator_interval_is_rejected() {
        let bounds = IndexKeyBounds {
            lower: Some(b"m".to_vec()),
            upper: Some(b"t".to_vec()),
        };
        assert!(bounds.admits(&node(&[b"m", b"s"])));
        assert!(!bounds.admits(&node(&[b"l", b"s"])));
        assert!(!bounds.admits(&node(&[b"m", b"t"])));
    }
}
