//! A consumer names only the public Host facade for both derive and trait.
pub use worth_query_host::facade::application_contribution::ChargedBytes;

#[derive(ChargedBytes)]
#[charged_bytes(crate = "worth_query_host::facade::application_contribution")]
pub struct RetainedResult {
    pub total: u64,
    pub labels: Vec<String>,
    pub resident_notes: Option<Box<ResidentNotes>>,
}

#[derive(ChargedBytes)]
#[charged_bytes(crate = "worth_query_host::facade::application_contribution")]
pub struct ResidentNotes {
    pub lines: Vec<String>,
}
