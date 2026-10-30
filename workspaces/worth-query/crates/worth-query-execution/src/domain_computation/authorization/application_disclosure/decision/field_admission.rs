use worth_foundational::facade::{AspectFieldLocator, FieldKey};

use super::super::contract::WorthQueryAdmittedApplicationDisclosureField;

pub(in crate::domain_computation) struct WorthQueryApplicationInternalProjectionAdmission<'a> {
    field: &'a FieldKey,
    admitted: Option<&'a WorthQueryAdmittedApplicationDisclosureField>,
}

pub(in crate::domain_computation) struct WorthQueryApplicationDisclosedProjectionAdmission;

impl<'a> WorthQueryApplicationInternalProjectionAdmission<'a> {
    pub(super) const fn public(field: &'a FieldKey) -> Self {
        Self {
            field,
            admitted: None,
        }
    }

    pub(super) fn governed(
        admitted: &'a WorthQueryAdmittedApplicationDisclosureField,
        field: &'a FieldKey,
    ) -> Option<Self> {
        admitted.admits_projection_field(field).then_some(Self {
            field,
            admitted: Some(admitted),
        })
    }

    pub(in crate::domain_computation) fn projection_fields(&self) -> Vec<FieldKey> {
        match self.admitted {
            None => vec![self.field.clone()],
            Some(admitted) if admitted.is_bound_scalar(self.field.as_str()) => {
                vec![self.field.clone()]
            }
            Some(admitted) => admitted
                .projection_mask()
                .paths()
                .iter()
                .flat_map(|path| path.fields())
                .filter(|field| *field == self.field)
                .cloned()
                .collect(),
        }
    }

    pub(in crate::domain_computation) const fn field_key(&self) -> &FieldKey {
        self.field
    }

    pub(in crate::domain_computation) fn admits_locator(
        &self,
        locator: &AspectFieldLocator,
    ) -> bool {
        locator.field_path().fields() == std::slice::from_ref(self.field)
    }
}

impl WorthQueryApplicationDisclosedProjectionAdmission {
    pub(super) const fn public() -> Self {
        Self
    }

    pub(super) fn governed(
        admitted: &WorthQueryAdmittedApplicationDisclosureField,
        field: &FieldKey,
    ) -> Option<Self> {
        admitted.admits_projection_field(field).then_some(Self)
    }
}
