use serde::{Deserialize, Serialize};

use super::maintenance::DerivedIndexMaintenanceWork;
use super::DerivedIndexGeneration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexMaintenanceOutcome {
    pub generations: Vec<DerivedIndexGeneration>,
    pub work: DerivedIndexMaintenanceWork,
}
