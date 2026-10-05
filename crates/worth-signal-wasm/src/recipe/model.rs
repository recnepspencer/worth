use serde::{Deserialize, Serialize};

use crate::expression::model::{ConditionSpec, Expr, IdentitySpec, SignalValue};
use worth_signal::facade::{ChangedRegion, PartitionSubscription};

mod family_scope;
pub use family_scope::RecipeFamilyReadScopeSpec;
pub(crate) use family_scope::RecipeFamilyScopeKeyRequirement;

pub type WasmAspectId = u8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AspectSelectionSpec {
    #[serde(default)]
    pub aspect: Option<WasmAspectId>,
    #[serde(default)]
    pub aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpec {
    pub id: String,
    #[serde(default)]
    pub initial: SignalValue,
    #[serde(default)]
    pub produces_aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyedSourceFamilySpec {
    #[serde(rename = "familyId")]
    pub family_id: String,
    #[serde(default)]
    pub initial: SignalValue,
    #[serde(default)]
    pub produces_aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSpec {
    pub id: String,
    #[serde(default)]
    pub reads: Vec<RecipeReadSpec>,
    pub expr: Expr,
    #[serde(default)]
    pub when: Option<ConditionSpec>,
    #[serde(default)]
    pub identity: Option<IdentitySpec>,
    #[serde(default)]
    pub produces_aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeReadSignalSpec {
    pub id: String,
    #[serde(default)]
    pub scope: Option<PartitionSubscription>,
    #[serde(flatten)]
    pub aspects: AspectSelectionSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeReadSpec {
    LegacyId(String),
    Signal(RecipeReadSignalSpec),
}

impl RecipeReadSpec {
    pub fn id(&self) -> &str {
        match self {
            Self::LegacyId(id) => id,
            Self::Signal(spec) => &spec.id,
        }
    }

    pub fn scope(&self) -> Option<&PartitionSubscription> {
        match self {
            Self::LegacyId(_) => None,
            Self::Signal(spec) => spec.scope.as_ref(),
        }
    }

    pub fn aspect_spec(&self) -> Option<&AspectSelectionSpec> {
        match self {
            Self::LegacyId(_) => None,
            Self::Signal(spec) => Some(&spec.aspects),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RecipeFamilyReadSpec {
    Signal {
        id: String,
        #[serde(default)]
        scope: Option<RecipeFamilyReadScopeSpec>,
        #[serde(flatten)]
        aspects: AspectSelectionSpec,
    },
    Keyed {
        #[serde(rename = "familyId")]
        family_id: String,
        #[serde(default)]
        scope: Option<RecipeFamilyReadScopeSpec>,
        #[serde(flatten)]
        aspects: AspectSelectionSpec,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyedRecipeFamilySpec {
    #[serde(rename = "familyId")]
    pub family_id: String,
    #[serde(default)]
    pub reads: Vec<RecipeFamilyReadSpec>,
    pub expr: Expr,
    #[serde(default)]
    pub when: Option<ConditionSpec>,
    #[serde(default)]
    pub identity: Option<IdentitySpec>,
    #[serde(default)]
    pub produces_aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TransactionOp {
    Set {
        id: String,
        value: SignalValue,
        #[serde(default)]
        aspect: Option<WasmAspectId>,
        #[serde(default)]
        aspects: Option<Vec<WasmAspectId>>,
    },
    SetWithRegions {
        id: String,
        value: SignalValue,
        #[serde(rename = "changedRegions", default)]
        changed_regions: Vec<ChangedRegion>,
        #[serde(default)]
        aspect: Option<WasmAspectId>,
        #[serde(default)]
        aspects: Option<Vec<WasmAspectId>>,
    },
    SetMany {
        values: Vec<SetValue>,
    },
    SetManyWithRegions {
        values: Vec<SetValueWithRegions>,
    },
    SetManyKeyed {
        #[serde(rename = "familyId")]
        family_id: String,
        values: Vec<KeyedSetValue>,
    },
    SetPackedGridRgba {
        #[serde(rename = "familyId")]
        family_id: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetValue {
    pub id: String,
    pub value: SignalValue,
    #[serde(default)]
    pub aspect: Option<WasmAspectId>,
    #[serde(default)]
    pub aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyedSetValue {
    pub key: String,
    pub value: SignalValue,
    #[serde(default)]
    pub aspect: Option<WasmAspectId>,
    #[serde(default)]
    pub aspects: Option<Vec<WasmAspectId>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetValueWithRegions {
    pub id: String,
    pub value: SignalValue,
    #[serde(default)]
    pub changed_regions: Vec<ChangedRegion>,
    #[serde(default)]
    pub aspect: Option<WasmAspectId>,
    #[serde(default)]
    pub aspects: Option<Vec<WasmAspectId>>,
}
