//! Quotes come from the concrete semantic owner's fields and portable roster.
use super::{BridgeSemanticDependencyCandidate, BridgeSemanticLocality, FundedMeaning, D};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use worth_foundational::facade::{
    AspectShape, AuthoritativeAspectChangeKind, PortableAspectContract,
};

fn add(total: &mut usize, bytes: usize) -> Result<(), D> {
    *total = total.checked_add(bytes).ok_or(D::SizeLimit)?;
    Ok(())
}
fn array<T>(count: usize) -> Result<usize, D> {
    std::alloc::Layout::array::<T>(count)
        .map(|layout| layout.size())
        .map_err(|_| D::SizeLimit)
}
fn fixed() -> Result<usize, D> {
    let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<FundedMeaning>())
        .map_err(|_| D::SizeLimit)?;
    Ok(layout.pad_to_align().size())
}
pub(super) fn capture(candidate: &BridgeSemanticDependencyCandidate) -> Result<u64, D> {
    let mut total = fixed()?;
    for text in [
        candidate.source_node_identity(),
        candidate.declared_graph_role(),
        candidate.source_stage_identity().unwrap_or(""),
    ] {
        add(&mut total, text.len())?;
    }
    let contract = candidate.contract();
    add(&mut total, contract.key().as_str().len())?;
    let fields = match contract.shape() {
        AspectShape::Struct(shape) => {
            for field in shape.fields() {
                add(&mut total, field.key().as_str().len())?;
            }
            shape.fields().len()
        }
        AspectShape::Scalar(_) | AspectShape::Reference(_) | AspectShape::Opaque(_) => 0,
    };
    add(
        &mut total,
        PortableAspectContract::decode_field_roster_storage_bytes(fields).ok_or(D::SizeLimit)?,
    )?;
    let paths = candidate.projection_mask().paths();
    add(&mut total, array::<Vec<String>>(paths.len())?)?;
    for path in paths {
        add(&mut total, array::<String>(path.fields().len())?)?;
        for field in path.fields() {
            add(&mut total, field.as_str().len())?;
        }
    }
    add(&mut total, candidate.binding().clone_allocation_bytes())?;
    match candidate.locality() {
        BridgeSemanticLocality::SourcePartition(role) => add(&mut total, role.as_str().len())?,
        BridgeSemanticLocality::SourceRecord
        | BridgeSemanticLocality::ManagedSourceRecord
        | BridgeSemanticLocality::WholeLogicalGraph => {}
    }
    add(
        &mut total,
        array::<AuthoritativeAspectChangeKind>(candidate.relevant_changes().len())?,
    )?;
    total.try_into().map_err(|_| D::SizeLimit)
}

#[derive(Default)]
struct ShapeQuote {
    sequence_elements: usize,
    nonempty_sequences: usize,
    string_bytes: usize,
}
struct Count<'a>(&'a mut ShapeQuote);
impl<'de> DeserializeSeed<'de> for Count<'_> {
    type Value = ();
    fn deserialize<T: serde::Deserializer<'de>>(self, input: T) -> Result<(), T::Error> {
        input.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Count<'_> {
    type Value = ();
    fn expecting(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("Bridge authored semantic fields")
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<(), E> {
        self.0.string_bytes = self
            .0
            .string_bytes
            .checked_add(text.len())
            .ok_or_else(|| E::custom("semantic string storage overflow"))?;
        Ok(())
    }
    fn visit_seq<T: SeqAccess<'de>>(self, mut sequence: T) -> Result<(), T::Error> {
        let mut nonempty = false;
        while sequence.next_element_seed(Count(&mut *self.0))?.is_some() {
            nonempty = true;
            self.0.sequence_elements = self
                .0
                .sequence_elements
                .checked_add(1)
                .ok_or_else(|| serde::de::Error::custom("semantic roster storage overflow"))?;
        }
        if nonempty {
            self.0.nonempty_sequences = self
                .0
                .nonempty_sequences
                .checked_add(1)
                .ok_or_else(|| serde::de::Error::custom("semantic sequence count overflow"))?;
        }
        Ok(())
    }
    fn visit_map<T: MapAccess<'de>>(self, mut map: T) -> Result<(), T::Error> {
        while map.next_key_seed(Count(&mut *self.0))?.is_some() {
            map.next_value_seed(Count(&mut *self.0))?;
        }
        Ok(())
    }
}
pub(super) fn decode(bytes: &[u8]) -> Result<u64, D> {
    let mut quote = ShapeQuote::default();
    let mut input = serde_json::Deserializer::from_slice(bytes);
    Count(&mut quote)
        .deserialize(&mut input)
        .map_err(|_| D::Malformed)?;
    input.end().map_err(|_| D::Malformed)?;
    // The actual largest schema slot bounds each sequence element. Growth and
    // the non-zero minimum capacity are separate bounds, including short Vecs.
    let largest = PortableAspectContract::decode_field_roster_storage_bytes(1)
        .ok_or(D::SizeLimit)?
        .max(std::mem::size_of::<Vec<String>>())
        .max(std::mem::size_of::<String>())
        .max(std::mem::size_of::<AuthoritativeAspectChangeKind>());
    let mut total = fixed()?;
    let slots = quote
        .sequence_elements
        .checked_mul(2)
        .and_then(|n| {
            quote
                .nonempty_sequences
                .checked_mul(8)
                .and_then(|minimum| n.checked_add(minimum))
        })
        .ok_or(D::SizeLimit)?;
    add(&mut total, slots.checked_mul(largest).ok_or(D::SizeLimit)?)?;
    add(&mut total, quote.string_bytes)?;
    total.try_into().map_err(|_| D::SizeLimit)
}

pub(super) fn retained(data: &super::Meaning, string_bytes: usize) -> Result<u64, D> {
    let mut total = fixed()?;
    let fields = data.contract.struct_field_count();
    // Portable's private Vec backing is quoted by its owner. The JSON decoder
    // has no exact size hint, so retain a checked growth/minimum-capacity bound.
    let capacity = if fields == 0 {
        0
    } else {
        fields
            .checked_mul(2)
            .and_then(|n| n.checked_add(4))
            .ok_or(D::SizeLimit)?
    };
    add(
        &mut total,
        PortableAspectContract::decode_field_roster_storage_bytes(capacity).ok_or(D::SizeLimit)?,
    )?;
    add(
        &mut total,
        array::<Vec<String>>(data.projection_paths.capacity())?,
    )?;
    for path in &data.projection_paths {
        add(&mut total, array::<String>(path.capacity())?)?;
    }
    add(
        &mut total,
        array::<AuthoritativeAspectChangeKind>(data.changes.capacity())?,
    )?;
    // All owned strings are decoded from disjoint spans of this JSON input.
    add(&mut total, string_bytes)?;
    total.try_into().map_err(|_| D::SizeLimit)
}
