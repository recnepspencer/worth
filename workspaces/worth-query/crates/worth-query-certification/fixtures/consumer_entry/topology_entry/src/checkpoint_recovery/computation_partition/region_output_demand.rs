//! A demand for the region output of the scope it names. Only the fixture's
//! courts demand region outputs this way; the producer binding they demand is
//! shared with the native ownership oracle.

use worth_query_host::facade::application_contribution::WorthQueryApplicationOutputDemand;

use super::output_producer::RegionOutputFamily;
use super::*;

#[derive(Clone)]
pub(super) struct RegionOutputDemand(pub(super) String);
impl<Schema: TopologySchemaBinding> WorthQueryApplicationOutputDemand<Schema>
    for RegionOutputDemand
{
    type OutputFamily = RegionOutputFamily;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}
