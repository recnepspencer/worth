//! Ordinary authenticated decisions borrow Query reads at their exact admitted product.

#[derive(Debug)]
pub enum DecisionQueryPreparationStop {
    MissingProduct,
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Native(worth_relational::facade::mvcc::CompanionPreflightStop),
}

impl std::fmt::Display for DecisionQueryPreparationStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for DecisionQueryPreparationStop {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Product(cause) => Some(cause),
            Self::Native(_) | Self::MissingProduct => None,
        }
    }
}

pub fn native_decision_cause<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a worth_relational::facade::mvcc::CompanionPreflightStop> {
    match error.downcast_ref::<DecisionQueryPreparationStop>()? {
        DecisionQueryPreparationStop::Native(cause) => Some(cause),
        _ => None,
    }
}
