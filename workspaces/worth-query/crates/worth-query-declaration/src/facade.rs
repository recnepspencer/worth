//! Public declaration-authority contract.

pub mod application_schema {
    pub use crate::application_schema::*;
}

pub mod application_aftermath {
    pub use crate::application_aftermath::*;
}

pub mod application_capability {
    pub use crate::application_capability::*;
}

pub mod application_query {
    pub use crate::application_query::*;
}

pub mod authentication {
    pub use crate::authentication::*;
}

pub mod authoring {
    pub use crate::authoring::*;
}

#[doc(hidden)]
pub mod foundation {
    pub use crate::authoring::*;
}

pub mod binding {
    pub use crate::binding::*;
}

pub mod branch {
    pub use crate::branch::*;
}

pub mod canonicalization {
    pub use crate::canonicalization::*;
}

pub mod collection {
    pub use crate::collection::*;
}

pub mod diagnostics {
    pub use crate::diagnostics::*;
}

pub mod domain_computation {
    pub use crate::domain_computation::*;
}

pub mod identity {
    pub use crate::identity::*;
}

pub mod portable_identity {
    pub use crate::portable_identity::*;
}

pub mod identity_authority {
    pub use crate::identity_authority::QueryCanonicalAuthority;
}

pub mod result_shape {
    pub use crate::result_shape::*;
}

pub mod schema_view {
    pub use crate::schema_basis_authority::QuerySchemaBasisAuthority;
    pub use crate::schema_view::*;
}

pub mod typed {
    pub use crate::schema_view::{QuerySchemaView, SchemaFieldView, SchemaRelationView};
    pub use crate::typed::*;
    pub use worth_foundational::facade::ScalarAspectType;
}

pub mod validation {
    pub use crate::validation::*;
}

pub mod view_declaration {
    pub use crate::view_declaration::*;
}
