pub use worth_query::facade::domain;

pub use super::{GeometryDomain, ReadFamily};

#[derive(Clone, Copy, Debug)]
pub struct CountVertices;

pub struct CountVerticesInput {
    pub minimum: Option<u64>,
}

impl domain::WorthQueryOperationInput for CountVerticesInput {
    fn parameters(&self) -> Vec<domain::WorthQueryOperationParameter<'_>> {
        self.minimum
            .map(|minimum| {
                vec![domain::WorthQueryOperationParameter::new(
                    "minimum",
                    domain::WorthQueryOperationParameterValue::U64(minimum),
                )]
            })
            .unwrap_or_default()
    }
}

impl domain::WorthQueryExecutableDomainOperation<GeometryDomain, ReadFamily> for CountVertices {
    type Input = CountVerticesInput;
    type Output = u64;
    type Publication = domain::WorthQueryTerminalOperation;
    type Execution = domain::WorthQueryDirectOperation;
}
