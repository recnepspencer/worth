// Pure owners derive resident descriptions without an Execution dependency.

pub(crate) use std::mem::size_of;
pub(crate) use worth_foundational::facade::ChargedBytes;

#[derive(serde::Serialize, ChargedBytes)]
#[charged_bytes(crate = "worth_foundational::facade")]
pub(crate) struct Record {
    pub(crate) label: String,
    // The boxed Vec is the subject: its box allocation is resident capacity.
    #[allow(clippy::box_collection)]
    #[serde(skip)]
    // This adversarial shape must count the boxed Vec header and spare capacity.
    #[expect(
        clippy::box_collection,
        reason = "the court verifies both nested heap owners, including a serde-skipped field"
    )]
    pub(crate) cache: Option<Box<Vec<String>>>,
}
