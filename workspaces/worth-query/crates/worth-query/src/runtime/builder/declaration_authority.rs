use crate::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};

const DECLARATION_AUTHORITY_DENIAL: &str =
    "declaration-authority runtimes do not own query execution";

/// A declaration-only composition root with no execution workspace surface.
///
/// ```compile_fail
/// use worth_query::facade::runtime::WorthQueryDeclarationAuthorityRuntime;
///
/// let declaration = WorthQueryDeclarationAuthorityRuntime::builder().build();
/// let _workspace = declaration.workspace("execution-is-not-declaration");
/// ```
pub struct WorthQueryDeclarationAuthorityRuntime {
    support_profile: WorthQueryRuntimeSupportProfile,
}

impl WorthQueryDeclarationAuthorityRuntime {
    pub fn builder() -> WorthQueryDeclarationAuthorityRuntimeBuilder {
        WorthQueryDeclarationAuthorityRuntimeBuilder
    }

    pub fn support_profile(&self) -> &WorthQueryRuntimeSupportProfile {
        &self.support_profile
    }
}

#[derive(Default)]
pub struct WorthQueryDeclarationAuthorityRuntimeBuilder;

impl WorthQueryDeclarationAuthorityRuntimeBuilder {
    pub fn build(self) -> WorthQueryDeclarationAuthorityRuntime {
        WorthQueryDeclarationAuthorityRuntime {
            support_profile: WorthQueryRuntimeSupportProfile::new(
                WorthQueryRuntimeFacadeFamily::ALL
                    .into_iter()
                    .map(|family| {
                        WorthQueryRuntimeFamilySupport::unsupported(
                            family,
                            DECLARATION_AUTHORITY_DENIAL,
                        )
                    }),
            ),
        }
    }
}
