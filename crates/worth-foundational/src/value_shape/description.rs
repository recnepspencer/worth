//! Descriptive portable grammar; Rust names and TypeId are not wire identity.
use serde::Serialize;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ValueShape {
    Primitive(ValuePrimitive),
    Sequence(Box<ValueShape>),
    Optional(Box<ValueShape>),
    Tuple(Vec<ValueShape>),
    Struct(Vec<ValueFieldShape>),
    Enum(Vec<ValueVariantShape>),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ValuePrimitive {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    String,
    Unit,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueFieldShape {
    name: String,
    shape: ValueShape,
}
impl ValueFieldShape {
    pub fn new(name: &'static str, shape: ValueShape) -> Self {
        Self {
            name: name.into(),
            shape,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn shape(&self) -> &ValueShape {
        &self.shape
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueVariantShape {
    name: String,
    payload: Option<ValueShape>,
}
impl ValueVariantShape {
    pub fn newtype(name: &'static str, shape: ValueShape) -> Self {
        Self {
            name: name.into(),
            payload: Some(shape),
        }
    }
    pub fn unit(name: &'static str) -> Self {
        Self {
            name: name.into(),
            payload: None,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn payload(&self) -> Option<&ValueShape> {
        self.payload.as_ref()
    }
}
