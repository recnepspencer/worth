use worth_store_physical_format::integrity_declarations::{
    families::root::{
        CURRENT_SELECTOR_INTEGRITY_DECLARATION, PREVIOUS_SELECTOR_INTEGRITY_DECLARATION,
        ROOT_MANIFEST_INTEGRITY_DECLARATION,
    },
    PhysicalIntegrityFormatDeclaration,
};

/// Declaration-only inputs for the first independent Phase 3 reader slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineIntegrityRootProtocolDeclarations {
    current_selector: PhysicalIntegrityFormatDeclaration,
    previous_selector: PhysicalIntegrityFormatDeclaration,
    root_manifest: PhysicalIntegrityFormatDeclaration,
}

pub const OFFLINE_INTEGRITY_ROOT_PROTOCOL_DECLARATIONS: OfflineIntegrityRootProtocolDeclarations =
    OfflineIntegrityRootProtocolDeclarations {
        current_selector: CURRENT_SELECTOR_INTEGRITY_DECLARATION,
        previous_selector: PREVIOUS_SELECTOR_INTEGRITY_DECLARATION,
        root_manifest: ROOT_MANIFEST_INTEGRITY_DECLARATION,
    };

impl OfflineIntegrityRootProtocolDeclarations {
    pub const fn current_selector(self) -> PhysicalIntegrityFormatDeclaration {
        self.current_selector
    }

    pub const fn previous_selector(self) -> PhysicalIntegrityFormatDeclaration {
        self.previous_selector
    }

    pub const fn root_manifest(self) -> PhysicalIntegrityFormatDeclaration {
        self.root_manifest
    }
}
