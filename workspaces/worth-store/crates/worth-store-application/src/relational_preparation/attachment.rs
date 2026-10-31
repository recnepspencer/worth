//! Attach one raw Native owner while retaining the same physical Home custody.
use super::*;
use worth_execution::ExecutionRequest;
use worth_proof::HomeAdmitted;

pub fn attach_relational(
    admitted: HomeAdmitted<crate::AdmittedApplicationHome>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<UnrecoveredApplicationHome, RelationalAttachmentRefused> {
    if let Err(cause) = request.consult() {
        return Err(RelationalAttachmentRefused::BeforeConstruction {
            cause,
            home: admitted,
        });
    }
    admitted.consume(|admitted| {
        let crate::AdmittedApplicationHome { context, definition,
            #[cfg(feature = "certification-test-authority")] read_observer,
        } = admitted;
        let serving = context.serving.clone();
        let placement = crate::open::physical_admission::placement();
        let selection = context.checkpoint.selection_source();
        #[cfg(not(feature = "certification-test-authority"))]
        let attached = worth_store_relational::UnrecoveredRelationalSection::attach_unrecovered_home(definition, serving, placement, selection);
        #[cfg(feature = "certification-test-authority")]
        let attached = match read_observer {
            Some(observer) => worth_store_relational::UnrecoveredRelationalSection::attach_unrecovered_home_with_read_observer_for_test(definition, serving, placement, selection, observer),
            None => worth_store_relational::UnrecoveredRelationalSection::attach_unrecovered_home(definition, serving, placement, selection),
        };
        match attached {
            Ok(relational) => Ok(UnrecoveredApplicationHome { relational, context }),
            Err(cause) => Err(RelationalAttachmentRefused::Construction(RelationalAttachmentFailedHome { cause, context })),
        }
    })
}
