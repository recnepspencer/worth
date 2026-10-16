//! Count actual boxed dependency nodes through the existing serde grammar.
use serde::{de::IgnoredAny, Deserialize, Deserializer};

#[derive(Deserialize)]
pub(super) struct Inventory {
    pub(super) version: u8,
    primary: NodeCount,
    support: Option<NodeCount>,
}

impl Inventory {
    pub(super) fn boxed_nodes(&self) -> usize {
        self.primary.0.saturating_sub(1)
            + self
                .support
                .as_ref()
                .map_or(0, |node| node.0.saturating_sub(1))
    }
}

struct NodeCount(usize);
impl<'de> Deserialize<'de> for NodeCount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Node {
            #[serde(rename = "primary")]
            _primary: IgnoredAny,
            #[serde(rename = "recorded_actor")]
            _recorded_actor: IgnoredAny,
            delegation: Option<Delegation>,
            #[serde(rename = "activation")]
            _activation: IgnoredAny,
        }
        #[derive(Deserialize)]
        enum Delegation {
            Root {
                #[serde(rename = "discovery")]
                _discovery: IgnoredAny,
            },
            Delegated {
                #[serde(rename = "discovery")]
                _discovery: IgnoredAny,
                #[serde(rename = "transition")]
                _transition: IgnoredAny,
                parent: NodeCount,
            },
        }
        let node = Node::deserialize(deserializer)?;
        Ok(Self(
            1 + match node.delegation {
                Some(Delegation::Delegated { parent, .. }) => parent.0,
                _ => 0,
            },
        ))
    }
}
