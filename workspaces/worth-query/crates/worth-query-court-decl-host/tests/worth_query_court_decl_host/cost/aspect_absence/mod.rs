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

mod classes;
use classes::*;

mod three {
    use super::*;
    const CLASSES: usize = 3;
    type Classes = (
        Option<classes::Class1>,
        Option<classes::Class2>,
        Option<classes::Class3>,
    );
    include!("schema.rs");
    include!("operation.rs");
    mod readback {
        include!("readback.rs");
    }
    include!("world.rs");
    mod class_access {
        include!("class_access.rs");
    }
}
mod six {
    use super::*;
    const CLASSES: usize = 6;
    type Classes = (
        Option<classes::Class1>,
        Option<classes::Class2>,
        Option<classes::Class3>,
        Option<classes::Class4>,
        Option<classes::Class5>,
        Option<classes::Class6>,
    );
    include!("schema.rs");
    include!("operation.rs");
    mod readback {
        include!("readback.rs");
    }
    include!("world.rs");
    mod class_access {
        include!("class_access.rs");
    }
}
mod twelve {
    use super::*;
    const CLASSES: usize = 12;
    type Classes = (
        (
            Option<classes::Class1>,
            Option<classes::Class2>,
            Option<classes::Class3>,
            Option<classes::Class4>,
            Option<classes::Class5>,
            Option<classes::Class6>,
            Option<classes::Class7>,
            Option<classes::Class8>,
        ),
        (
            Option<classes::Class9>,
            Option<classes::Class10>,
            Option<classes::Class11>,
            Option<classes::Class12>,
        ),
    );
    include!("schema.rs");
    include!("operation.rs");
    mod readback {
        include!("readback.rs");
    }
    include!("world.rs");
    mod class_access {
        include!("class_access.rs");
    }
}

mod court;
