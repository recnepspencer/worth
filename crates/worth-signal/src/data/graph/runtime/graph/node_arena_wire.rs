use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::persistent_vector::PersistentVector;

use super::{
    CompactionState, NodeArena, NodeColdData, NodeDefinitionData, NodeHotData, NodeWarmData, Slot,
};

// The wire's warm rows retain their existing shape. Separating live definition
// storage must not erase installed meaning when reading an existing graph.
#[derive(Deserialize)]
struct ArenaWire {
    nodes: PersistentPagedVector<Slot>,
    hot: PersistentPagedVector<Option<NodeHotData>>,
    warm: Vec<WarmWire>,
    cold: PersistentPagedVector<Option<Box<NodeColdData>>>,
    free_list: PersistentVector<u32>,
    active_nodes: u32,
    #[serde(default)]
    compaction: CompactionState,
}

#[derive(Deserialize)]
struct WarmWire {
    #[serde(flatten)]
    definition: NodeDefinitionData,
    #[serde(flatten)]
    evaluation: NodeWarmData,
}

#[derive(Serialize)]
struct ArenaWireRef<'a> {
    nodes: &'a PersistentPagedVector<Slot>,
    hot: &'a PersistentPagedVector<Option<NodeHotData>>,
    warm: WarmRows<'a>,
    cold: &'a PersistentPagedVector<Option<Box<NodeColdData>>>,
    free_list: &'a PersistentVector<u32>,
    active_nodes: u32,
    compaction: &'a CompactionState,
}

struct WarmRows<'a>(&'a NodeArena);

#[derive(Serialize)]
struct WarmWireRef<'a> {
    #[serde(flatten)]
    definition: &'a NodeDefinitionData,
    #[serde(flatten)]
    evaluation: &'a NodeWarmData,
}

impl Serialize for WarmRows<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0.definitions.len() != self.0.warm.len() {
            return Err(serde::ser::Error::custom(
                "definition and evaluation lanes differ",
            ));
        }
        let mut sequence = serializer.serialize_seq(Some(self.0.warm.len()))?;
        for (definition, evaluation) in self.0.definitions.iter().zip(self.0.warm.iter()) {
            sequence.serialize_element(&WarmWireRef {
                definition,
                evaluation,
            })?;
        }
        sequence.end()
    }
}

impl Serialize for NodeArena {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ArenaWireRef {
            nodes: &self.nodes,
            hot: &self.hot,
            warm: WarmRows(self),
            cold: &self.cold,
            free_list: &self.free_list,
            active_nodes: self.active_nodes,
            compaction: &self.compaction,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NodeArena {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ArenaWire::deserialize(deserializer)?;
        let mut definitions = PersistentPagedVector::new();
        let mut warm = PersistentPagedVector::new();
        for row in wire.warm {
            definitions.push_back(row.definition);
            warm.push_back(row.evaluation);
        }
        Ok(Self {
            definitions,
            nodes: wire.nodes,
            hot: wire.hot,
            warm,
            cold: wire.cold,
            free_list: wire.free_list,
            free_slots: Default::default(),
            active_nodes: wire.active_nodes,
            compaction: wire.compaction,
            retained_node_ledger: None,
            retained_node_custody: None,
            retained_seed_custody: None,
        })
    }
}
