//! Allocation-free closure inspection. Every recursive cycle must consume
//! structural depth; a local admitted seed cannot hide an unsupported child.
use super::{
    ValueCodecDenial as D, ValueDecodeAdmission, ValuePrimitive, ValueShapeBody,
    ValueShapeDefinition, ValueShapeDepth,
};
use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

type InspectionCache = RwLock<BTreeMap<(TypeId, bool), ValueShapeDepth>>;
static INSPECTIONS: OnceLock<InspectionCache> = OnceLock::new();

struct Frame<'a> {
    identity: TypeId,
    depth: usize,
    parent: Option<&'a Frame<'a>>,
}

impl ValueShapeDefinition {
    pub fn inspect(self) -> Result<ValueShapeDepth, D> {
        cached_inspection(self, &mut |_, _| Ok(()), false)
    }
    pub fn inspect_admitted(
        self,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<ValueShapeDepth, D> {
        cached_inspection(self, admission, true)
    }
    pub fn supports_admission(self) -> bool {
        self.inspect_admitted(&mut |_, _| Ok(())).is_ok()
    }
}

// Static grammar identity and admission posture are the entire equivalence
// basis. Only successful closure proofs are retained; request refusals never are.
fn cached_inspection(
    definition: ValueShapeDefinition,
    admission: &mut ValueDecodeAdmission<'_>,
    admitted: bool,
) -> Result<ValueShapeDepth, D> {
    let cache = INSPECTIONS.get_or_init(InspectionCache::default);
    let key = (definition.identity, admitted);
    if let Some(depth) = cache
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&key)
        .copied()
    {
        admission.admit(1, 0)?;
        return Ok(depth);
    }
    let depth = inspect(definition, None, 0, admission, admitted)?;
    cache
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(key, depth);
    Ok(depth)
}

fn inspect(
    definition: ValueShapeDefinition,
    parent: Option<&Frame<'_>>,
    depth: usize,
    admission: &mut ValueDecodeAdmission<'_>,
    admitted: bool,
) -> Result<ValueShapeDepth, D> {
    admission.admit(1, 0)?;
    if admitted && !definition.admitted {
        return Err(D::UnsupportedAdmission);
    }
    let mut ancestor = parent;
    while let Some(frame) = ancestor {
        admission.admit(1, 0)?;
        if frame.identity == definition.identity {
            return if depth > frame.depth {
                Ok(ValueShapeDepth::Recursive)
            } else {
                Err(D::UnsupportedShape)
            };
        }
        ancestor = frame.parent;
    }
    let frame = Frame {
        identity: definition.identity,
        depth,
        parent,
    };
    let structural = !matches!(
        definition.body,
        ValueShapeBody::Primitive(_) | ValueShapeBody::Transparent(_)
    );
    let next = depth
        .checked_add(usize::from(structural))
        .ok_or(D::UnsupportedShape)?;
    let mut extent = ValueShapeDepth::Finite(usize::from(structural));
    if let ValueShapeBody::Optional(edge) = definition.body
        && nullable(edge.resolve(), Some(&frame), admission)?
    {
        return Err(D::UnsupportedShape);
    }
    // Unit variants have no child visit to pay for their inventory traversal.
    match definition.body {
        ValueShapeBody::Enum(variants) => admission.admit(variants.len(), 0)?,
        ValueShapeBody::TaggedEnum { variants, .. } => admission.admit(variants.len(), 0)?,
        _ => {}
    }
    let mut child = |edge: super::definition::ValueShapeEdge| {
        let value = inspect(edge.resolve(), Some(&frame), next, admission, admitted)?;
        extent = match (extent, value) {
            (ValueShapeDepth::Finite(a), ValueShapeDepth::Finite(b)) => {
                ValueShapeDepth::Finite(a.max(b + usize::from(structural)))
            }
            _ => ValueShapeDepth::Recursive,
        };
        Ok::<_, D>(())
    };
    match definition.body {
        ValueShapeBody::Primitive(_) => {}
        ValueShapeBody::Transparent(edge) | ValueShapeBody::Sequence(edge) => child(edge)?,
        ValueShapeBody::Optional(edge) => child(edge)?,
        ValueShapeBody::Tuple(edges) => {
            for &edge in edges {
                child(edge)?;
            }
        }
        ValueShapeBody::Struct(fields) => {
            for field in fields {
                child(field.value)?;
            }
        }
        ValueShapeBody::Enum(variants) => {
            for variant in variants {
                if let Some(edge) = variant.payload {
                    child(edge)?;
                }
            }
        }
        ValueShapeBody::TaggedEnum { variants, .. } => {
            for variant in variants {
                for field in variant.fields {
                    child(field.value)?;
                }
            }
        }
    }
    Ok(extent)
}

fn nullable(
    definition: ValueShapeDefinition,
    parent: Option<&Frame<'_>>,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<bool, D> {
    admission.admit(1, 0)?;
    match definition.body {
        ValueShapeBody::Primitive(kind) => Ok(kind == ValuePrimitive::Unit),
        ValueShapeBody::Optional(_) => Ok(true),
        ValueShapeBody::Transparent(edge) => {
            let mut ancestor = parent;
            while let Some(frame) = ancestor {
                admission.admit(1, 0)?;
                if frame.identity == definition.identity {
                    return Err(D::UnsupportedShape);
                }
                ancestor = frame.parent;
            }
            let frame = Frame {
                identity: definition.identity,
                depth: 0,
                parent,
            };
            nullable(edge.resolve(), Some(&frame), admission)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod cost {
    use super::*;
    use crate::value_shape::{
        CanonicalValue, ValueDecodeBounds, ValueSeedDelegate, ValueShapeEdge,
    };

    struct TupleGrammar<const N: usize>;
    impl<const N: usize> ValueSeedDelegate for TupleGrammar<N> {
        type Value = Vec<String>;
        const SUPPORTS_ADMISSION: bool = true;
        const WIRE_NULLABLE: bool = false;
        const SHAPE_BODY: ValueShapeBody =
            ValueShapeBody::Tuple(&[ValueShapeEdge::value::<String>(); N]);
        fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D> {
            if value.len() != N {
                return Err(D::Malformed);
            }
            value.validate_bounds(bounds)
        }
        fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
            decoder: Decoder,
            bounds: ValueDecodeBounds,
            admission: Option<&mut ValueDecodeAdmission<'_>>,
        ) -> Result<Self::Value, Decoder::Error> {
            let value =
                <crate::value_shape::ValueDelegate<Vec<String>> as ValueSeedDelegate>::deserialize(
                    decoder, bounds, admission,
                )?;
            if value.len() != N {
                return Err(serde::de::Error::custom(D::Malformed));
            }
            Ok(value)
        }
    }

    #[test]
    fn admitted_static_shape_lookup_stays_flat_as_the_grammar_grows() {
        fn work<const N: usize>() -> usize {
            let definition = ValueShapeDefinition::from_delegate::<TupleGrammar<N>>();
            definition.inspect_admitted(&mut |_, _| Ok(())).unwrap();
            let mut work = 0;
            definition
                .inspect_admitted(&mut |charge, _| {
                    work += charge;
                    Ok(())
                })
                .unwrap();
            work
        }
        let baseline = work::<128>();
        for actual in [work::<256>(), work::<512>()] {
            assert!(
                (actual as f64) / (baseline as f64) < 1.5,
                "repeated shape traversal: {baseline} -> {actual}"
            );
        }
    }
}
