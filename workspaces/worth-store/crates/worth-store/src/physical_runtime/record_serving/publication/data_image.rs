use crate::physical_runtime::integrity::IntegrityAdmittedResidentPageBasis;

pub(in crate::physical_runtime::record_serving) struct ExistingDataFrameImage {
    bytes: Vec<u8>,
    admitted_prior_basis: IntegrityAdmittedResidentPageBasis,
}

impl ExistingDataFrameImage {
    pub(in crate::physical_runtime::record_serving) fn new(
        bytes: Vec<u8>,
        admitted_prior_basis: IntegrityAdmittedResidentPageBasis,
    ) -> Option<Self> {
        let coordinate = admitted_prior_basis.coordinate();
        if bytes.len() != coordinate.length() as usize {
            return None;
        }
        Some(Self {
            bytes,
            admitted_prior_basis,
        })
    }

    pub(in crate::physical_runtime::record_serving) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::physical_runtime::record_serving) const fn admitted_prior_basis(
        &self,
    ) -> IntegrityAdmittedResidentPageBasis {
        self.admitted_prior_basis
    }

    pub(in crate::physical_runtime::record_serving) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
