use std::any::TypeId;

/// Structural posture of one domain-owned change.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationChangePosture {
    Preserve,
    Replace,
    Split,
    Merge,
    CreateDelete,
    Rewire,
    Reparent,
    Reconstruct,
}

/// Domain-owned change meaning attached to one canonical program action.
///
/// The operation handler and installed invariants still own the actual candidate.
pub trait ApplicationChangeShape: Sized + 'static {
    const IDENTITY: &'static str;
    const POSTURE: ApplicationChangePosture;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationChangeShapeDeclaration {
    identity: &'static str,
    posture: ApplicationChangePosture,
    shape_type: TypeId,
}

impl ApplicationChangeShapeDeclaration {
    pub(crate) fn of<Shape: ApplicationChangeShape>() -> Self {
        Self {
            identity: Shape::IDENTITY,
            posture: Shape::POSTURE,
            shape_type: TypeId::of::<Shape>(),
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn posture(&self) -> ApplicationChangePosture {
        self.posture
    }

    pub const fn shape_type(&self) -> TypeId {
        self.shape_type
    }
}
