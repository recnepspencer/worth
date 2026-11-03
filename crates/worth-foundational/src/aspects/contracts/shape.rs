use crate::aspects::structs::StructAspectShape;
use crate::values::ScalarAspectType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpaqueAspectType {
    Token,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceAspectType {
    Entity,
}

/// Native content belongs to format-bound Struct fields. A whole-aspect
/// content form cannot be constructed.
/// ```compile_fail,E0599
/// let _ = worth_foundational::AspectShape::Content;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AspectShape {
    Scalar(ScalarAspectType),
    Struct(StructAspectShape),
    Opaque(OpaqueAspectType),
    Reference(ReferenceAspectType),
}
