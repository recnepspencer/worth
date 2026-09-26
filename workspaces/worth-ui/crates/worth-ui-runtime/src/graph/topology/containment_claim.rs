use crate::capability::MosaicSizingContractId;
use crate::declaration::UiDeclarationContainmentIntent;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiGraphContainmentClaim {
    RootPage,
    PageSet {
        page_set_name: Box<str>,
    },
    Region {
        region_name: Box<str>,
    },
    Mosaic {
        mosaic_name: Box<str>,
        sizing_contract_id: Option<MosaicSizingContractId>,
    },
    LocalComposition {
        local_composition_name: Box<str>,
    },
    Control {
        control_name: Box<str>,
    },
    DiagnosticSurface {
        diagnostic_surface_name: Box<str>,
    },
}

impl UiGraphContainmentClaim {
    pub(crate) fn from_declaration_intent(
        containment_intent: &UiDeclarationContainmentIntent,
        mosaic_sizing_contract_id: Option<MosaicSizingContractId>,
    ) -> Self {
        match containment_intent {
            UiDeclarationContainmentIntent::RootTopology => Self::RootPage,
            UiDeclarationContainmentIntent::DeclaredPageSetMembership { page_set_name } => {
                Self::PageSet {
                    page_set_name: page_set_name.clone(),
                }
            }
            UiDeclarationContainmentIntent::DeclaredRegionMembership { region_name } => {
                Self::Region {
                    region_name: region_name.clone(),
                }
            }
            UiDeclarationContainmentIntent::DeclaredMosaicMembership { mosaic_name } => {
                Self::Mosaic {
                    mosaic_name: mosaic_name.clone(),
                    sizing_contract_id: mosaic_sizing_contract_id,
                }
            }
            UiDeclarationContainmentIntent::DeclaredLocalCompositionMembership {
                local_composition_name,
            } => Self::LocalComposition {
                local_composition_name: local_composition_name.clone(),
            },
            UiDeclarationContainmentIntent::DeclaredControlAttachment { control_name } => {
                Self::Control {
                    control_name: control_name.clone(),
                }
            }
            UiDeclarationContainmentIntent::DeclaredDiagnosticAttachment {
                diagnostic_surface_name,
            } => Self::DiagnosticSurface {
                diagnostic_surface_name: diagnostic_surface_name.clone(),
            },
        }
    }

    pub const fn is_root_page(&self) -> bool {
        matches!(self, Self::RootPage)
    }

    pub fn claim_name(&self) -> Option<&str> {
        match self {
            Self::RootPage => None,
            Self::PageSet { page_set_name } => Some(page_set_name),
            Self::Region { region_name } => Some(region_name),
            Self::Mosaic { mosaic_name, .. } => Some(mosaic_name),
            Self::LocalComposition {
                local_composition_name,
            } => Some(local_composition_name),
            Self::Control { control_name } => Some(control_name),
            Self::DiagnosticSurface {
                diagnostic_surface_name,
            } => Some(diagnostic_surface_name),
        }
    }

    pub fn mosaic_sizing_contract_id(&self) -> Option<&MosaicSizingContractId> {
        match self {
            Self::Mosaic {
                sizing_contract_id, ..
            } => sizing_contract_id.as_ref(),
            Self::RootPage
            | Self::PageSet { .. }
            | Self::Region { .. }
            | Self::LocalComposition { .. }
            | Self::Control { .. }
            | Self::DiagnosticSurface { .. } => None,
        }
    }
}
