//! Cold portable description construction. Acyclic grammar stays a tree;
//! recursive references use lexical ordinals, never Rust type identities.
use super::{
    ValueFieldShape, ValueShape, ValueShapeBody as Body, ValueShapeDefinition,
    ValueTaggedVariantShape, ValueVariantShape,
};
use std::any::TypeId;

struct Frame {
    identity: TypeId,
    binding: usize,
    referenced: bool,
}

impl ValueShapeDefinition {
    pub fn materialize(self) -> ValueShape {
        let mut stack = Vec::new();
        let mut shape = expand(self, &mut stack, &mut 0);
        renumber(&mut shape, &mut Vec::new(), &mut 0);
        shape
    }
}

fn expand(
    definition: ValueShapeDefinition,
    stack: &mut Vec<Frame>,
    next: &mut usize,
) -> ValueShape {
    if let Some(frame) = stack
        .iter_mut()
        .find(|frame| frame.identity == definition.identity)
    {
        frame.referenced = true;
        return ValueShape::Reference {
            binding: frame.binding,
        };
    }
    let binding = *next;
    *next += 1;
    stack.push(Frame {
        identity: definition.identity,
        binding,
        referenced: false,
    });
    let shape = match definition.body {
        Body::Primitive(kind) => ValueShape::Primitive(kind),
        Body::Transparent(edge) => expand(edge.resolve(), stack, next),
        Body::Sequence(edge) => ValueShape::Sequence(Box::new(expand(edge.resolve(), stack, next))),
        Body::Optional(edge) => ValueShape::Optional(Box::new(expand(edge.resolve(), stack, next))),
        Body::Tuple(edges) => ValueShape::Tuple(
            edges
                .iter()
                .map(|edge| expand(edge.resolve(), stack, next))
                .collect(),
        ),
        Body::Struct(definitions) => ValueShape::Struct(fields(definitions, stack, next)),
        Body::Enum(variants) => ValueShape::Enum(
            variants
                .iter()
                .map(|variant| match variant.payload {
                    Some(edge) => ValueVariantShape::newtype(
                        variant.name,
                        expand(edge.resolve(), stack, next),
                    ),
                    None => ValueVariantShape::unit(variant.name),
                })
                .collect(),
        ),
        Body::TaggedEnum { tag, variants } => ValueShape::TaggedEnum {
            tag: tag.into(),
            variants: variants
                .iter()
                .map(|variant| ValueTaggedVariantShape {
                    name: variant.name.into(),
                    fields: fields(variant.fields, stack, next),
                })
                .collect(),
        },
    };
    if stack.pop().expect("active grammar frame").referenced {
        ValueShape::Recursive {
            binding,
            body: Box::new(shape),
        }
    } else {
        shape
    }
}
fn fields(
    definitions: &[super::ValueFieldDefinition],
    stack: &mut Vec<Frame>,
    next: &mut usize,
) -> Vec<ValueFieldShape> {
    definitions
        .iter()
        .map(|field| ValueFieldShape::new(field.name, expand(field.value.resolve(), stack, next)))
        .collect()
}
fn renumber(shape: &mut ValueShape, bindings: &mut Vec<(usize, usize)>, next: &mut usize) {
    match shape {
        ValueShape::Recursive { binding, body } => {
            bindings.push((*binding, *next));
            *binding = *next;
            *next += 1;
            renumber(body, bindings, next);
            bindings.pop();
        }
        ValueShape::Reference { binding } => {
            *binding = bindings
                .iter()
                .rev()
                .find(|(old, _)| old == binding)
                .expect("reference is lexically bound")
                .1;
        }
        ValueShape::Sequence(child) | ValueShape::Optional(child) => {
            renumber(child, bindings, next)
        }
        ValueShape::Tuple(children) => {
            for child in children {
                renumber(child, bindings, next);
            }
        }
        ValueShape::Struct(fields) => {
            for field in fields {
                renumber(&mut field.shape, bindings, next);
            }
        }
        ValueShape::Enum(variants) => {
            for variant in variants {
                if let Some(payload) = &mut variant.payload {
                    renumber(payload, bindings, next);
                }
            }
        }
        ValueShape::TaggedEnum { variants, .. } => {
            for variant in variants {
                for field in &mut variant.fields {
                    renumber(&mut field.shape, bindings, next);
                }
            }
        }
        ValueShape::Primitive(_) => {}
    }
}
