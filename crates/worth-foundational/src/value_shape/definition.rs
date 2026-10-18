//! Static grammar edges. Function pointers close recursive type definitions;
//! compiler-local identities are used only while inspecting/materializing them.
use super::{CanonicalValue, ValuePrimitive};
use std::any::TypeId;

#[derive(Clone, Copy)]
pub struct ValueShapeEdge(fn() -> ValueShapeDefinition);
impl ValueShapeEdge {
    pub const fn value<Value: CanonicalValue>() -> Self {
        Self(ValueShapeDefinition::of::<Value>)
    }
    pub const fn delegate<Delegate: super::delegates::ValueSeedDelegate>() -> Self {
        Self(ValueShapeDefinition::delegate::<Delegate>)
    }
    pub(super) fn resolve(self) -> ValueShapeDefinition {
        (self.0)()
    }
}

#[derive(Clone, Copy)]
pub enum ValueShapeBody {
    Primitive(ValuePrimitive),
    Transparent(ValueShapeEdge),
    Sequence(ValueShapeEdge),
    Optional(ValueShapeEdge),
    Tuple(&'static [ValueShapeEdge]),
    Struct(&'static [ValueFieldDefinition]),
    Enum(&'static [ValueVariantDefinition]),
    TaggedEnum {
        tag: &'static str,
        variants: &'static [ValueTaggedVariantDefinition],
    },
}

#[derive(Clone, Copy)]
pub struct ValueFieldDefinition {
    pub name: &'static str,
    pub value: ValueShapeEdge,
}
#[derive(Clone, Copy)]
pub struct ValueVariantDefinition {
    pub name: &'static str,
    pub payload: Option<ValueShapeEdge>,
}
#[derive(Clone, Copy)]
pub struct ValueTaggedVariantDefinition {
    pub name: &'static str,
    pub fields: &'static [ValueFieldDefinition],
}

#[derive(Clone, Copy)]
pub struct ValueShapeDefinition {
    pub(super) identity: TypeId,
    pub(super) admitted: bool,
    pub(super) body: ValueShapeBody,
}
impl ValueShapeDefinition {
    pub fn of<Value: CanonicalValue>() -> Self {
        Self {
            identity: TypeId::of::<Value>(),
            admitted: Value::SUPPORTS_ADMISSION,
            body: Value::SHAPE_BODY,
        }
    }
    pub(super) fn delegate<Delegate: super::delegates::ValueSeedDelegate>() -> Self {
        Self {
            identity: TypeId::of::<Delegate>(),
            admitted: Delegate::SUPPORTS_ADMISSION,
            body: Delegate::SHAPE_BODY,
        }
    }
}
