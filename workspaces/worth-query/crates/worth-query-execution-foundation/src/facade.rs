//! Explicit internal contract for Query execution foundations.

pub mod equality_type_id {
    pub use crate::equality_type_id::*;
}

pub mod execution_digest {
    pub use crate::execution_digest::*;
}

pub mod relational_snapshot_release {
    pub use crate::relational_snapshot_release::*;
}

pub mod domain_computation {
    pub mod application_contract_admission {
        pub use crate::domain_computation::application_contract_admission::*;
    }
    pub mod application_outcome_identity {
        pub use crate::domain_computation::application_outcome_identity::*;
    }
    pub mod artifact_identity {
        pub use crate::domain_computation::artifact_identity::*;
    }
    pub mod evidence_material {
        pub use crate::domain_computation::evidence_material::*;
    }
    pub mod managed_run_vocabulary {
        pub use crate::domain_computation::managed_run_vocabulary::*;
    }
}

pub mod primary_graph {
    pub mod admitted_capture {
        pub use crate::domain_computation::primary_graph::admitted_capture::*;
    }
    pub mod application_branch {
        pub use crate::domain_computation::primary_graph::application_branch::*;
    }
    pub mod exact_basis_access {
        pub use crate::domain_computation::primary_graph::exact_basis_access::*;
    }
    pub mod index_maintenance_budget {
        pub use crate::domain_computation::primary_graph::index_maintenance_budget::*;
    }
    pub mod output_binding_identity {
        pub use crate::domain_computation::primary_graph::output_binding_identity::*;
    }
    pub mod output_family_identity {
        pub use crate::domain_computation::primary_graph::output_family_identity::*;
    }
    pub mod principal_key {
        pub use crate::domain_computation::primary_graph::principal_key::*;
    }
    pub mod projection_read_origin {
        pub use crate::domain_computation::primary_graph::projection_read_origin::*;
    }
    pub mod publication_request_denial {
        pub use crate::domain_computation::primary_graph::publication_request_denial::*;
    }
    pub mod request_allocation_control {
        pub use crate::domain_computation::primary_graph::request_allocation_control::*;
    }
    pub mod request_local {
        pub use crate::domain_computation::primary_graph::request_local::*;
    }
    pub mod value_decode {
        pub use crate::domain_computation::primary_graph::value_decode::*;
    }
}
