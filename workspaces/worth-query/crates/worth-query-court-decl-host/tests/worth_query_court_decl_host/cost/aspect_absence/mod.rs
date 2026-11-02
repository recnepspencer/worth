//! Declared class count does not add reads to a selected native field.
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_query::*, application_schema::*,
    authentication::*, *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, operator_identity::*,
};

macro_rules! fixture {
    ($name:ident, $count:literal) => {
        mod $name {
            use super::*;
            const CLASSES: usize = $count;
            include!("schema.rs");
            include!("operation.rs");
            mod readback {
                include!("readback.rs");
            }
            include!("world.rs");
        }
    };
}
fixture!(three, 3);
fixture!(six, 6);
fixture!(twelve, 12);

mod court;
