use serde::{Deserialize, Serialize};

use super::binding_denial::RelationalMergeRequestBindingDenial;
use crate::merge::data::requests::normalized::{
    RelationalMergeCorrespondencePosture, RelationalMergeSchemaReconciliationPosture,
    RelationalMergeTopologyIntent,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeRequestNormalizationDenial {
    OwnerBinding(RelationalMergeRequestBindingDenial),
    UnsupportedCorrespondencePosture {
        posture: RelationalMergeCorrespondencePosture,
    },
    UnsupportedSchemaReconciliationPosture {
        posture: RelationalMergeSchemaReconciliationPosture,
    },
    UnsupportedTopologyIntent {
        intent: RelationalMergeTopologyIntent,
    },
}
