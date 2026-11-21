use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleCardinality;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{CommitResult, EntityReference};
use super::{
    OutputRoleUse, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture,
};
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationEffectEntity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::effect_program::output_correspondence::candidate::*;
pub mod authoring;
