//! Serialized directory mutation input shared by the installed Catalog profiles.
#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Retire,
    Create,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Replace {
    pub edit: Edit,
}
