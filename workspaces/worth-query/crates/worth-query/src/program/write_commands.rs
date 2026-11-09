use std::collections::BTreeMap;

use crate::runtime::WorthQueryAspectTouch;

use super::error::WorthQueryProgramError;
use super::expressions::WorthQueryValueExpr;
use super::validation::expect_string;
use super::values::WorthQueryProgramValue;

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAdmittedAspectValueTemplate {
    aspect_touch: WorthQueryAspectTouch,
    value: WorthQueryValueExpr,
}

impl WorthQueryAdmittedAspectValueTemplate {
    pub fn new(aspect_touch: WorthQueryAspectTouch, value: WorthQueryValueExpr) -> Self {
        Self {
            aspect_touch,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorthQueryWriteCommandTemplate {
    InsertAspects {
        collection: String,
        aspects: Vec<WorthQueryAdmittedAspectValueTemplate>,
    },
    UpdateAspect {
        entity_identity: WorthQueryValueExpr,
        aspect_touch: WorthQueryAspectTouch,
        value: WorthQueryValueExpr,
    },
    Delete {
        entity_identity: WorthQueryValueExpr,
    },
}

impl WorthQueryWriteCommandTemplate {
    pub(crate) fn bind(
        &self,
        inputs: &BTreeMap<String, WorthQueryProgramValue>,
    ) -> Result<crate::runtime::WorthQueryWriteCommand, WorthQueryProgramError> {
        match self {
            Self::InsertAspects {
                collection,
                aspects,
            } => Ok(crate::runtime::WorthQueryWriteCommand::InsertAspects {
                collection: crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
                    "write-command-declared",
                    collection,
                )?,
                aspects: aspects
                    .iter()
                    .map(|aspect| {
                        crate::runtime::WorthQueryAuthoredAspectMutation::new_set(
                            aspect.aspect_touch.clone(),
                            aspect.value.evaluate(inputs)?.foundational_scalar_value()?,
                        )
                        .map_err(|error| WorthQueryProgramError::new(error.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                symbolic_aspect_references: Vec::new(),
                metadata: crate::runtime::WorthQueryMutationMetadata::default(),
                naming_intent: None,
                continuity_intent: None,
                symbolic_target_reference: None,
            }),
            Self::UpdateAspect {
                entity_identity,
                aspect_touch,
                value,
            } => Ok(crate::runtime::WorthQueryWriteCommand::UpdateAspect {
                entity_identity: crate::memory_workspace::admit_authored_entity_label(
                    expect_string(entity_identity.evaluate(inputs)?, "entity_identity")?,
                )?,
                aspect: crate::runtime::WorthQueryAuthoredAspectMutation::new_set(
                    aspect_touch.clone(),
                    value.evaluate(inputs)?.foundational_scalar_value()?,
                )
                .map_err(|error| WorthQueryProgramError::new(error.to_string()))?,
            }),
            Self::Delete { entity_identity } => {
                Ok(crate::runtime::WorthQueryWriteCommand::Delete {
                    entity_identity: crate::memory_workspace::admit_authored_entity_label(
                        expect_string(entity_identity.evaluate(inputs)?, "entity_identity")?,
                    )?,
                })
            }
        }
    }
}
