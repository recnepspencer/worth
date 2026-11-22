pub mod application_inspection {
    pub mod rich;
    pub mod source_original;
    pub mod target;
    pub use rich::WorthQueryApplicationMaterializationRequest;
    pub use target::WorthQueryApplicationInspectionTarget;
}

pub mod application_observation {
    pub mod declared_query;
    pub use declared_query::WorthQueryApplicationQueryObservation;
}

pub use application_inspection::{
    WorthQueryApplicationInspectionTarget, WorthQueryApplicationMaterializationRequest,
};
pub use application_observation::WorthQueryApplicationQueryObservation;
