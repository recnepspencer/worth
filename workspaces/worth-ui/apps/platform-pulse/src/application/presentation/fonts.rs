use std::sync::Arc;
use worth_ui::facade::app::{
    UiApplicationFontFaceDefinition, UiApplicationFontLicenseRecord,
    UiApplicationFontPackDefinition, UiFontCollectionGeneration, UiFontFamilyStack, UiFontSlant,
    UiGlobalFontCollection,
};

pub(in crate::application) struct PulseFonts {
    pub(in crate::application) collection: Arc<UiGlobalFontCollection>,
    pub(super) headings: UiFontFamilyStack,
    pub(super) body: UiFontFamilyStack,
}

impl PulseFonts {
    pub(in crate::application) fn admit() -> Self {
        let (profile, _) = UiGlobalFontCollection::admit_qualified_profile()
            .expect("embedded qualified Pulse font profile");
        let (collection, receipt, _) = profile
            .register_application_pack(
                UiFontCollectionGeneration::new(2).unwrap(),
                UiApplicationFontPackDefinition {
                    name: Arc::from("Platform Pulse typography"),
                    faces: Box::new([
                        UiApplicationFontFaceDefinition {
                            family: Arc::from("Gelasio"),
                            bytes: Arc::from(
                                include_bytes!("../../../assets/fonts/Gelasio.ttf").as_slice(),
                            ),
                            face_index: 0,
                            weight: 400,
                            width_milli_percent: 100_000,
                            slant: UiFontSlant::Upright,
                            license: UiApplicationFontLicenseRecord {
                                identifier: Arc::from("OFL-1.1"),
                                notice: Arc::from(include_str!(
                                    "../../../assets/fonts/OFL-Gelasio.txt"
                                )),
                            },
                        },
                        UiApplicationFontFaceDefinition {
                            family: Arc::from("Roboto"),
                            bytes: Arc::from(
                                include_bytes!("../../../assets/fonts/Roboto.ttf").as_slice(),
                            ),
                            face_index: 0,
                            weight: 400,
                            width_milli_percent: 100_000,
                            slant: UiFontSlant::Upright,
                            license: UiApplicationFontLicenseRecord {
                                identifier: Arc::from("OFL-1.1"),
                                notice: Arc::from(include_str!(
                                    "../../../assets/fonts/OFL-Roboto.txt"
                                )),
                            },
                        },
                    ]),
                },
            )
            .expect("bundled OFL dashboard families pass font qualification");
        Self {
            collection: Arc::new(collection),
            headings: UiFontFamilyStack::new(Box::new([receipt.family("Gelasio").unwrap()]))
                .unwrap(),
            body: UiFontFamilyStack::new(Box::new([receipt.family("Roboto").unwrap()])).unwrap(),
        }
    }
}
