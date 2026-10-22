//! Allocation-free closure inspection. Every recursive cycle must consume
//! structural depth; a local admitted seed cannot hide an unsupported child.
use super::{
    ValueCodecDenial as D, ValueDecodeAdmission, ValuePrimitive, ValueShapeBody,
    ValueShapeDefinition, ValueShapeDepth,
};
use std::any::TypeId;

struct Frame<'a> {
    identity: TypeId,
    depth: usize,
    parent: Option<&'a Frame<'a>>,
}

impl ValueShapeDefinition {
    pub fn inspect(self) -> Result<ValueShapeDepth, D> {
        inspect(self, None, 0, &mut |_, _| Ok(()), false)
    }
    pub fn inspect_admitted(
        self,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<ValueShapeDepth, D> {
        inspect(self, None, 0, admission, true)
    }
    pub fn supports_admission(self) -> bool {
        self.inspect_admitted(&mut |_, _| Ok(())).is_ok()
    }
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
