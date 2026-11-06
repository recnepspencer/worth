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
        let crate::AdmittedApplicationHome {
            context,
            definition,
            native_source,
        } = admitted;
        let serving = context.serving.clone();
        let placement = crate::open::physical_admission::placement();
        let selection = context.checkpoint.selection_source();
        let attached = match native_source {
            crate::home_context::RelationalHomeOpeningInput::Fresh(prepared) => {
                worth_store_relational::UnrecoveredRelationalSection::attach_fresh_home(
                    definition,
                    serving,
                    placement,
                    selection,
                    prepared.with_home_transition_authority(
                        context.transition.authority(),
                        context.transition.required(),
                    ),
                )
            }
            crate::home_context::RelationalHomeOpeningInput::Existing(prepared) => {
                worth_store_relational::UnrecoveredRelationalSection::attach_prepared_home(
                    definition,
                    serving,
                    placement,
                    selection,
                    prepared.with_home_transition_authority(
                        context.transition.authority(),
                        context.transition.required(),
                    ),
                )
            }
        };
        match attached {
            Ok(relational) => Ok(UnrecoveredApplicationHome {
                relational,
                context,
            }),
            Err(cause) => Err(RelationalAttachmentRefused::Construction(
                RelationalAttachmentFailedHome { cause, context },
            )),
        }
    })
}
