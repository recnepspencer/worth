//! A single-pass borrowed node visitor builds only validated, funded AVL nodes.
use super::*;

pub(super) struct Node<'c, 'r, 'scope, 'authority, 'source> {
    pub(super) context: &'c mut Readmission<'r, 'scope, 'authority>,
    pub(super) previous: &'c mut Option<RouteKey<'source>>,
    pub(super) depth: usize,
}
impl<'de> DeserializeSeed<'de> for Node<'_, '_, '_, '_, 'de> {
    type Value = Option<Arc<PositionNode>>;
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        self.context
            .charge(1)
            .map_err(|cause| self.context.stop(cause))?;
        decoder.deserialize_option(self)
    }
}
impl<'de> Visitor<'de> for Node<'_, '_, '_, '_, 'de> {
    type Value = Option<Arc<PositionNode>>;
    fn expecting(&self, output: &mut std::fmt::Formatter) -> std::fmt::Result {
        output.write_str("an absent or ordered checkpoint route node")
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
    fn visit_some<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
        if self.depth == MAXIMUM_HEIGHT {
            return Err(serde::de::Error::custom(
                "checkpoint route height exceeds image bounds",
            ));
        }
        decoder.deserialize_struct(
            "CheckpointOutputPositionNode",
            &[
                "family",
                "source",
                "partition",
                "publication",
                "left",
                "right",
            ],
            Record(self),
        )
    }
}

struct Record<'c, 'r, 'scope, 'authority, 'source>(Node<'c, 'r, 'scope, 'authority, 'source>);
impl<'de> Visitor<'de> for Record<'_, '_, '_, '_, 'de> {
    type Value = Option<Arc<PositionNode>>;
    fn expecting(&self, output: &mut std::fmt::Formatter) -> std::fmt::Result {
        output.write_str("one ordered checkpoint route node")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
        let seed = self.0;
        let family: &'de str = field(&mut input, "family")?;
        let source: EntityId = field(&mut input, "source")?;
        let partition: [u8; 32] = field(&mut input, "partition")?;
        let route_key = (family, source, partition);
        let (tag, commit): (u8, CommitId) = field(&mut input, "publication")?;
        if !(seed.context.contains)(commit) {
            return Err(serde::de::Error::custom(
                "checkpoint route is beyond its root prefix",
            ));
        }
        let publication = match tag {
            0 => RequiredOutputRoutePosition::Published(commit),
            1 => RequiredOutputRoutePosition::CodecRetired(commit),
            _ => {
                return Err(serde::de::Error::custom(
                    "unknown checkpoint route position tag",
                ))
            }
        };
        key(&mut input, "left")?;
        let left = input.next_value_seed(Node {
            context: seed.context,
            previous: seed.previous,
            depth: seed.depth + 1,
        })?;
        // In-order comparison visits each family's bytes at most twice. Carrying
        // ancestor bounds would repeatedly charge a long ancestor's text.
        let comparison_work = family
            .len()
            .checked_add(seed.previous.map_or(0, |key| key.0.len()))
            .and_then(|n| n.checked_add(64))
            .ok_or_else(|| seed.context.stop::<A::Error>(Refusal::CapacityOverflow))?;
        seed.context
            .charge(comparison_work as u64)
            .map_err(|cause| seed.context.stop(cause))?;
        if seed.previous.is_some_and(|previous| route_key <= previous) {
            return Err(serde::de::Error::custom(
                "checkpoint route keys are not strictly ordered",
            ));
        }
        *seed.previous = Some(route_key);
        key(&mut input, "right")?;
        let right = input.next_value_seed(Node {
            context: seed.context,
            previous: seed.previous,
            depth: seed.depth + 1,
        })?;
        finish(&mut input)?;
        let left_height = super::super::height(&left);
        let right_height = super::super::height(&right);
        if left_height.abs_diff(right_height) > 1 {
            return Err(serde::de::Error::custom(
                "checkpoint route tree is not balanced",
            ));
        }
        let height = left_height
            .max(right_height)
            .checked_add(1)
            .ok_or_else(|| seed.context.stop::<A::Error>(Refusal::CapacityOverflow))?;
        let route_memory = seed
            .context
            .reserve::<PublicationRoute>(family.len())
            .map_err(|cause| seed.context.stop(cause))?;
        let node_memory = seed
            .context
            .reserve::<PositionNode>(0)
            .map_err(|cause| seed.context.stop(cause))?;
        let route = Arc::new(PublicationRoute {
            family: family.to_owned(),
            source,
            partition,
            _funding: route_memory,
        });
        Ok(Some(Arc::new(PositionNode {
            route,
            publication,
            left,
            right,
            height,
            _funding: node_memory,
        })))
    }
}
