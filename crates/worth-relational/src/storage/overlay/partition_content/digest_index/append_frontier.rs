//! A right spine plus opaque left siblings is sufficient to extend a
//! monotonically indexed history, without retaining its earlier values.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoryAppendFrontier {
    siblings: Vec<HistoryFrontierSibling>,
    last: Option<(u128, [u8; 32])>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryFrontierSibling {
    key: u128,
    bit: u32,
    digest: [u8; 32],
    parent_bit: u32,
}

impl DigestIndex {
    pub(in crate::storage::overlay::partition_content) fn append_frontier(
        &self,
        last_value: Option<[u8; 32]>,
    ) -> Option<HistoryAppendFrontier> {
        let mut siblings = Vec::new();
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            if let Some(children) = &node.children {
                siblings.push(HistoryFrontierSibling {
                    key: children[0].key,
                    bit: children[0].bit,
                    digest: children[0].digest,
                    parent_bit: node.bit,
                });
                current = Some(&children[1]);
            } else {
                assert_eq!(node.bit, 128, "history frontier reaches its last value");
                let value = last_value.or_else(|| {
                    self.history_last_value
                        .as_ref()
                        .filter(|last| last.key == node.key)
                        .map(|last| last.value)
                })?;
                if hash(1, &[&node.key.to_be_bytes(), &value]) != node.digest {
                    return None;
                }
                return Some(HistoryAppendFrontier {
                    siblings,
                    last: Some((node.key, value)),
                });
            }
        }
        last_value.is_none().then_some(HistoryAppendFrontier {
            siblings,
            last: None,
        })
    }

    pub(in crate::storage::overlay::partition_content) fn from_append_frontier(
        frontier: &HistoryAppendFrontier,
    ) -> Option<Self> {
        let Some((key, value)) = frontier.last else {
            return frontier.siblings.is_empty().then(Self::default);
        };
        // The cold prefix need not fit this process's address space, but its
        // carried next position must be representable before any append.
        key.checked_add(1)?;
        let mut node = Arc::new(Node {
            key,
            bit: 128,
            digest: hash(1, &[&key.to_be_bytes(), &value]),
            children: None,
            nodes: 1,
        });
        for sibling in frontier.siblings.iter().rev() {
            if sibling.parent_bit >= node.bit
                || sibling.bit <= sibling.parent_bit
                || (sibling.key ^ node.key).leading_zeros() != sibling.parent_bit
                || direction(sibling.key, sibling.parent_bit) != 0
                || direction(node.key, sibling.parent_bit) != 1
            {
                return None;
            }
            let left = Arc::new(Node {
                key: sibling.key,
                bit: sibling.bit,
                digest: sibling.digest,
                children: None,
                nodes: 1,
            });
            let mut parent = Node {
                key: sibling.key,
                bit: sibling.parent_bit,
                digest: [0; 32],
                children: Some([left, node]),
                nodes: 0,
            };
            refresh(&mut parent);
            node = Arc::new(parent);
        }
        Some(Self {
            root: Some(node),
            history_last_value: Some(Arc::new(HistoryLastValue { key, value })),
        })
    }

    pub(in crate::storage::overlay::partition_content) fn next_history_index(&self) -> u128 {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            if let Some(children) = &node.children {
                current = Some(&children[1]);
            } else {
                assert_eq!(node.bit, 128, "history frontier includes its last key");
                return node.key.checked_add(1).expect("history index fits u128");
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_binds_the_last_history_key_even_without_a_parent_path() {
        let value = sha2::Sha256::digest(b"one historical version").into();
        let mut full = DigestIndex::default();
        full.set(0, Some(value));
        let mut frontier = full.append_frontier(Some(value)).unwrap();
        frontier.last.as_mut().unwrap().0 = 1;
        let forged = DigestIndex::from_append_frontier(&frontier).unwrap();
        assert_ne!(
            forged.digest(),
            full.digest(),
            "a relabelled append frontier must fail the committed leaf check"
        );
    }

    #[test]
    fn history_frontier_refuses_an_exhausted_carried_position() {
        let value = sha2::Sha256::digest(b"one historical version").into();
        let mut full = DigestIndex::default();
        full.set(0, Some(value));
        let mut frontier = full.append_frontier(Some(value)).unwrap();
        frontier.last.as_mut().unwrap().0 = u128::MAX;
        assert!(DigestIndex::from_append_frontier(&frontier).is_none());
    }

    #[test]
    fn history_frontier_preserves_commitment_and_future_appends() {
        let mut full = DigestIndex::default();
        for index in 0..1_024_u128 {
            full.set(
                index,
                Some(sha2::Sha256::digest(index.to_be_bytes()).into()),
            );
        }
        let last_value = sha2::Sha256::digest(1_023_u128.to_be_bytes()).into();
        let frontier = full.append_frontier(Some(last_value)).unwrap();
        assert!(frontier.siblings.len() < 32);
        let mut head = DigestIndex::from_append_frontier(&frontier).unwrap();
        assert_eq!(head.digest(), full.digest());
        assert_eq!(head.next_history_index(), 1_024);
        for index in 1_024..1_100_u128 {
            let digest = sha2::Sha256::digest(index.to_be_bytes()).into();
            full.set(index, Some(digest));
            head.set(index, Some(digest));
            assert_eq!(head.digest(), full.digest());
        }
        assert!(head.allocation_bytes() < full.allocation_bytes() / 4);
        let recaptured = head
            .append_frontier(None)
            .expect("head retains its last value opening without old history");
        assert_eq!(
            DigestIndex::from_append_frontier(&recaptured)
                .unwrap()
                .digest(),
            full.digest()
        );
    }
}
