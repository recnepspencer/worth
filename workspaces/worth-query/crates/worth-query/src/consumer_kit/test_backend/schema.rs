use std::collections::{BTreeMap, BTreeSet};

use crate::memory_workspace::WorthQueryAspect;
use crate::runtime::WorthQueryAspectTouch;
use worth_foundational::facade::{AspectContract, AspectShape, CanonicalFieldPath, FieldKey};

use super::error::{WorthQueryTestBackendError, WorthQueryTestBackendErrorKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTestBackendSchema {
    collection: String,
    contracts: BTreeMap<worth_foundational::facade::AspectKey, AspectContract>,
    aspects: Vec<WorthQueryTestBackendAspect>,
}

impl WorthQueryTestBackendSchema {
    pub fn single_collection(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            contracts: BTreeMap::new(),
            aspects: Vec::new(),
        }
    }

    pub fn aspect_contract(
        mut self,
        contract: AspectContract,
    ) -> Result<Self, WorthQueryTestBackendError> {
        match self.contracts.get(contract.key()) {
            Some(installed) if installed == &contract => return Ok(self),
            Some(_) => {
                return Err(WorthQueryTestBackendError::new(
                    WorthQueryTestBackendErrorKind::ConflictingAspectContract,
                    format!(
                        "test backend schema declares conflicting Foundational contracts for aspect `{}`",
                        contract.key().as_str()
                    ),
                ));
            }
            None => {}
        }
        self.contracts.insert(contract.key().clone(), contract);
        Ok(self)
    }

    pub fn aspect_contracts(
        mut self,
        contracts: impl IntoIterator<Item = AspectContract>,
    ) -> Result<Self, WorthQueryTestBackendError> {
        for contract in contracts {
            self = self.aspect_contract(contract)?;
        }
        Ok(self)
    }

    pub fn aspect(
        self,
        label: impl Into<String>,
        projection_field_path_text: impl Into<String>,
    ) -> Result<Self, WorthQueryTestBackendError> {
        let label = label.into();
        ensure_non_blank(
            &label,
            WorthQueryTestBackendErrorKind::BlankAspectLabel,
            "test backend aspect label may not be blank",
        )?;
        let aspect_touch = WorthQueryAspectTouch::from_authoring_ingress_text(label.clone())
            .map_err(|error| {
                WorthQueryTestBackendError::new(
                    WorthQueryTestBackendErrorKind::InvalidAspectLabel,
                    format!(
                        "test backend aspect label `{label}` is not a native aspect touch: {error}"
                    ),
                )
            })?;
        let projection_field_path_text = projection_field_path_text.into();
        ensure_non_blank(
            &projection_field_path_text,
            WorthQueryTestBackendErrorKind::BlankProjectionPath,
            "test backend projection field path authoring text may not be blank",
        )?;
        let native_field_path =
            canonical_field_path_from_authoring_text(&projection_field_path_text).ok_or_else(
                || {
                    WorthQueryTestBackendError::new(
                        WorthQueryTestBackendErrorKind::InvalidProjectionPath,
                        format!(
                            "test backend projection field path authoring text `{projection_field_path_text}` is not a canonical field path"
                        ),
                    )
                },
            )?;
        self.native_aspect_mapping(aspect_touch, native_field_path)
    }

    pub fn native_aspect_mapping(
        mut self,
        aspect_touch: WorthQueryAspectTouch,
        native_field_path: CanonicalFieldPath,
    ) -> Result<Self, WorthQueryTestBackendError> {
        self.validate_mapping_contract(&aspect_touch)?;
        self.aspects.push(WorthQueryTestBackendAspect {
            touch: aspect_touch,
            native_field_path,
        });
        self.validate()?;
        Ok(self)
    }

    pub fn collection(&self) -> &str {
        &self.collection
    }

    pub fn aspects(&self) -> impl Iterator<Item = (&WorthQueryAspectTouch, &CanonicalFieldPath)> {
        self.aspects
            .iter()
            .map(|aspect| (&aspect.touch, &aspect.native_field_path))
    }

    pub fn contracts(&self) -> impl Iterator<Item = &AspectContract> {
        self.contracts.values()
    }

    pub(crate) fn contract(&self, key: &worth_foundational::facade::AspectKey) -> &AspectContract {
        self.contracts
            .get(key)
            .expect("validated test schemas retain every mapped aspect contract")
    }

    pub(crate) fn memory_aspects(
        &self,
    ) -> Result<Vec<WorthQueryAspect>, WorthQueryTestBackendError> {
        self.validate()?;
        Ok(self
            .aspects()
            .map(|(touch, path)| {
                WorthQueryAspect::from_native_field_path(touch.clone(), path.clone())
            })
            .collect())
    }

    pub(crate) fn validate(&self) -> Result<(), WorthQueryTestBackendError> {
        ensure_non_blank(
            self.collection(),
            WorthQueryTestBackendErrorKind::BlankCollectionName,
            "test backend collection name may not be blank",
        )?;
        let mut labels = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for aspect in &self.aspects {
            self.validate_mapping_contract(&aspect.touch)?;
            ensure_unique(
                labels.insert(aspect.touch.clone()),
                WorthQueryTestBackendErrorKind::DuplicateAspectLabel,
                format!(
                    "test backend schema declares duplicate admitted aspect touch `{}`",
                    reporting_projection_from_admitted_touch(&aspect.touch)
                ),
            )?;
            ensure_unique(
                paths.insert(aspect.native_field_path.clone()),
                WorthQueryTestBackendErrorKind::DuplicateProjectionPath,
                format!(
                    "test backend schema declares duplicate native field path `{}`",
                    terminal_projection_from_field_path(&aspect.native_field_path)
                ),
            )?;
        }
        if self.aspects.is_empty() {
            return Err(WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::EmptyAspectSet,
                "test backend schema must declare at least one aspect",
            ));
        }
        Ok(())
    }

    fn validate_mapping_contract(
        &self,
        touch: &WorthQueryAspectTouch,
    ) -> Result<(), WorthQueryTestBackendError> {
        let contract = self
            .contracts
            .get(touch.native_aspect_key())
            .ok_or_else(|| {
                WorthQueryTestBackendError::new(
                    WorthQueryTestBackendErrorKind::MissingAspectContract,
                    format!(
                        "test backend mapping `{}` has no installed Foundational aspect contract",
                        reporting_projection_from_admitted_touch(touch)
                    ),
                )
            })?;
        let Some(path) = touch.native_field_path() else {
            return Ok(());
        };
        let fields = path.fields();
        if fields.len() != 1 {
            return Err(WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::NestedAspectFieldUnsupported,
                format!(
                    "test backend mapping `{}` addresses a nested field, but Foundational struct contracts currently declare one field level",
                    reporting_projection_from_admitted_touch(touch)
                ),
            ));
        }
        let AspectShape::Struct(shape) = contract.shape() else {
            return Err(WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::AspectFieldRequiresStructContract,
                format!(
                    "test backend mapping `{}` addresses a field beneath non-struct aspect `{}`",
                    reporting_projection_from_admitted_touch(touch),
                    contract.key().as_str()
                ),
            ));
        };
        if shape.field(&fields[0]).is_none() {
            return Err(WorthQueryTestBackendError::new(
                WorthQueryTestBackendErrorKind::UndeclaredAspectField,
                format!(
                    "test backend mapping `{}` addresses field `{}` absent from Foundational contract `{}`",
                    reporting_projection_from_admitted_touch(touch),
                    fields[0].as_str(),
                    contract.key().as_str()
                ),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorthQueryTestBackendAspect {
    touch: WorthQueryAspectTouch,
    native_field_path: CanonicalFieldPath,
}

fn ensure_non_blank(
    value: &str,
    kind: WorthQueryTestBackendErrorKind,
    message: impl Into<String>,
) -> Result<(), WorthQueryTestBackendError> {
    if value.trim().is_empty() {
        return Err(WorthQueryTestBackendError::new(kind, message));
    }
    Ok(())
}

fn ensure_unique(
    unique: bool,
    kind: WorthQueryTestBackendErrorKind,
    message: impl Into<String>,
) -> Result<(), WorthQueryTestBackendError> {
    if !unique {
        return Err(WorthQueryTestBackendError::new(kind, message));
    }
    Ok(())
}

fn canonical_field_path_from_authoring_text(path: &str) -> Option<CanonicalFieldPath> {
    CanonicalFieldPath::new(
        path.split('.')
            .map(FieldKey::new)
            .collect::<Option<Vec<_>>>()?,
    )
}

fn terminal_projection_from_field_path(field_path: &CanonicalFieldPath) -> String {
    field_path
        .fields()
        .iter()
        .map(FieldKey::as_str)
        .collect::<Vec<_>>()
        .join(".")
}

fn reporting_projection_from_admitted_touch(touch: &WorthQueryAspectTouch) -> String {
    touch.admitted_touch_digest_part()
}
