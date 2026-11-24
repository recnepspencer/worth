use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectFieldTargetRejectionReason {
    NestedFieldPath,
    UndeclaredAspect,
    FieldPathNotAdmittedByAspectBinding,
}

impl AspectFieldTargetRejectionReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NestedFieldPath => "nested field path",
            Self::UndeclaredAspect => "undeclared aspect",
            Self::FieldPathNotAdmittedByAspectBinding => {
                "field path not admitted by aspect binding"
            }
        }
    }
}
