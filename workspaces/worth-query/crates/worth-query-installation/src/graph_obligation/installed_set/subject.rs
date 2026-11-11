use crate::application_query::WorthQueryInstalledApplicationQueryIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledGraphObligationSubjectKind {
    ApplicationInspection,
    ApplicationManagedIndex,
    ApplicationQuery,
    ApplicationOperation,
    ApplicationResource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryInstalledGraphObligationSubject {
    ApplicationInspection {
        name: String,
        purpose: String,
    },
    ApplicationManagedIndex {
        name: String,
        parent_binding: String,
    },
    ApplicationQuery {
        name: String,
        identity: WorthQueryInstalledApplicationQueryIdentity,
    },
    ApplicationOperation {
        name: String,
        input_type: String,
    },
    ApplicationResource {
        name: String,
        purpose: String,
    },
}

impl WorthQueryInstalledGraphObligationSubject {
    pub(super) fn identity_fields(&self) -> (&'static str, &str, Option<&str>) {
        match self {
            Self::ApplicationInspection { name, purpose } => {
                ("application-inspection", name, Some(purpose))
            }
            Self::ApplicationManagedIndex {
                name,
                parent_binding,
            } => ("application-managed-index", name, Some(parent_binding)),
            Self::ApplicationQuery { name, .. } => ("application-query", name, None),
            Self::ApplicationOperation { name, input_type } => {
                ("application-operation", name, Some(input_type))
            }
            Self::ApplicationResource { name, purpose } => {
                ("application-resource", name, Some(purpose))
            }
        }
    }
}
