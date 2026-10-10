//! Reclamation compares component meaning and the admitted basis description.
use std::sync::Arc;

use crate::retention::unique_component_pin::ExactComponentBasis;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ComponentBasisOrder {
    Relational {
        reference: Arc<[u8]>,
        version: u64,
        root: u64,
        schema: [u8; 32],
        visibility: [u8; 32],
    },
    Signal {
        reference: Arc<[u8]>,
    },
}

impl ComponentBasisOrder {
    pub(super) fn from_admitted(component: ExactComponentBasis<'_>) -> Self {
        match component {
            ExactComponentBasis::Relational(basis) => {
                let descriptor = basis.descriptor();
                Self::Relational {
                    reference: descriptor.reference().canonical_encoding().into(),
                    version: descriptor.truth_version().as_u64(),
                    root: descriptor.root_identity(),
                    schema: descriptor.schema_commitment(),
                    visibility: descriptor.visibility_commitment(),
                }
            }
            ExactComponentBasis::Signal(basis) => Self::Signal {
                reference: basis.observation().canonical_encoding().into(),
            },
        }
    }
}
