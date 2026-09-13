use super::super::super::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};

pub(in crate::domain_computation::primary_graph::application_contribution) fn validate_required_producer(
    identity: &str,
    conditional_owner: &str,
    producer_owner: Option<&str>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let Some(producer_owner) = producer_owner else {
        return Err(denial(DenialKind::MissingProducerBinding, identity));
    };
    if producer_owner != conditional_owner {
        return Err(denial(DenialKind::ForeignProducerBinding, identity));
    }
    Ok(())
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}

#[cfg(test)]
mod tests {
    use super::{validate_required_producer, DenialKind};

    #[test]
    fn conditional_dependencies_reject_missing_and_foreign_producers() {
        let missing = validate_required_producer("body.initial", "cad", None).unwrap_err();
        assert_eq!(missing.kind(), DenialKind::MissingProducerBinding);

        let foreign =
            validate_required_producer("body.initial", "cad", Some("analysis")).unwrap_err();
        assert_eq!(foreign.kind(), DenialKind::ForeignProducerBinding);

        validate_required_producer("body.initial", "cad", Some("cad")).unwrap();
    }
}
