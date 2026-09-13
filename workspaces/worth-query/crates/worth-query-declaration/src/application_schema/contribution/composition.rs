use crate::application_schema::ApplicationSchema;

/// The root-owned contribution list used by declaration and host composition.
pub trait ApplicationSchemaComposition: ApplicationSchema {
    type Contributions;
}
