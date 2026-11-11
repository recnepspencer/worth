//! One attached Home proof relates the original bootstrap to its current receiver.
use super::*;
use worth_foundational::facade::{CanonicalDigestId, HomeProgramTransitionInstallationBinding};
use worth_proof::AdmittedHomeProgramTransitionSuccessor;

pub(super) fn expected_binding<Schema: ApplicationSchema>(
    native: &RelationalRuntime,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    selected: &ApplicationProgramRevision,
    transition: Option<
        &AdmittedHomeProgramTransitionSuccessor<HomeProgramTransitionInstallationBinding>,
    >,
) -> Result<codec::BootstrapInstallationBinding, Denial> {
    let current = installed.binding_identity();
    match transition {
        None => Ok(codec::BootstrapInstallationBinding::from_installed(
            &current,
        )),
        Some(transition) => {
            native
                .admit_home_program_transition_installation(transition)
                .map_err(|cause| {
                    denial(format!(
                        "Home installation transition proof refused: {cause:?}"
                    ))
                })?;
            let binding = transition.binding();
            if binding.successor()
                != [
                    *current.package_identity(),
                    *current.schema_identity(),
                    CanonicalDigestId::new(*selected.as_bytes()),
                ]
            {
                return Err(denial("Home installation transition does not name this current package/schema/program"));
            }
            let [package, schema] = binding.origin();
            Ok(codec::BootstrapInstallationBinding { package, schema })
        }
    }
}
