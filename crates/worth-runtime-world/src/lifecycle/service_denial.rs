/// Separates weak-owner loss from an available service's domain denial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeWorldServiceDenial<D> {
    OwnerUnavailable(super::RuntimeWorldOwnerUnavailable),
    Denied(D),
}
impl<D> From<super::RuntimeWorldOwnerUnavailable> for RuntimeWorldServiceDenial<D> {
    fn from(value: super::RuntimeWorldOwnerUnavailable) -> Self {
        Self::OwnerUnavailable(value)
    }
}
