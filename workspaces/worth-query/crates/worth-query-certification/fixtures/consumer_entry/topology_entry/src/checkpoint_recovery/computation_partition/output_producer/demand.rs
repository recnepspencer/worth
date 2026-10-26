// Consumer demand for a region output.
/// A demand for the region output of the scope it names.
#[derive(Clone)]
pub(super) struct RegionOutputDemand(pub(super) String);
impl<Schema: TopologySchemaBinding> application_contribution::WorthQueryApplicationOutputDemand<Schema>
    for RegionOutputDemand
{
    type OutputFamily = RegionOutputFamily;
    fn source_intent(&self) -> PlanarRead {
        PlanarRead {
            body_key: self.0.clone(),
        }
    }
}
