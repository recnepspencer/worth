//! Prefix probe meaning is independent of any catalog schema.
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Probe {
    pub prefix: String,
}
