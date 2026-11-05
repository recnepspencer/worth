//! Nominal alias input shared by the real prefix query bindings.
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct FindPrefixAlias {
    pub directory: String,
    pub prefix: String,
}
impl FindPrefixAlias {
    pub fn scope(input: Self) -> String {
        input.directory
    }
}
