//! The bounded frontier keeps its work bound and its caller's stop distinct.
#[derive(Debug)]
pub enum BoundedFrontierReadStop<Limit, Admission> {
    Limit(Limit),
    Admission(Admission),
}
