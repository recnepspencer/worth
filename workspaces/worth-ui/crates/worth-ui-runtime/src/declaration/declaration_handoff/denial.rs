use crate::declaration::{
    UiAspectContractAdmissionDenial, UiDeclarationFamilyAdmissionDenial,
    UiDeclarationStructuralSemanticsAdmissionDenial, UiDeclaredPostureAdmissionDenial,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiDeclarationGraphHandoffDenial {
    ComponentReferenceNotAdmitted,
    AppearanceRoleAttachmentNotAdmitted,
    AspectContractNotAdmitted {
        denial: UiAspectContractAdmissionDenial,
    },
    FamilyNotAdmitted {
        denial: UiDeclarationFamilyAdmissionDenial,
    },
    StructuralSemanticsNotAdmitted {
        denial: UiDeclarationStructuralSemanticsAdmissionDenial,
    },
    DeclaredPostureNotAdmitted {
        denial: UiDeclaredPostureAdmissionDenial,
    },
}
