use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AspectKey, AuthoritativePatchConstructionDenial};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityAuthoritativeAspectStateDenial {
    MissingAspectContract {
        aspect_key: AspectKey,
    },
    PatchConstructionDenied {
        denial: AuthoritativePatchConstructionDenial,
    },
}
