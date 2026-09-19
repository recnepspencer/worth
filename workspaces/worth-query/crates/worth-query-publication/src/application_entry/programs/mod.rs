//! Application-program operations bound to one admitted request and product
//! occurrence.

mod outcome;
mod preparation;
mod request;

pub use outcome::{WorthQueryBranchAdoptionPublicationOutcome, WorthQueryPerformedBranchAdoption};
pub use preparation::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramAdoptionRequestWithRequirements, WorthQueryPreparedBranchAdoption,
};
pub use request::{
    WorthQueryApplicationProgramAdoptionRequest, WorthQueryApplicationProgramsRequest,
};
