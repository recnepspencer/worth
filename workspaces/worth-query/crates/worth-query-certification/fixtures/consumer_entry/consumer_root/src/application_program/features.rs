use super::*;

pub(super) type ConsumerFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<ConsumerSchema, PlanarOutputFeature>,
        ApplicationFeatureList<
            ApplicationFeatureRef<ConsumerSchema, PlanarFinalOutputFeature>,
            ApplicationFeatureList<
                ApplicationFeatureRef<ConsumerSchema, PlanarAlternateFinalOutputFeature>,
                ApplicationFeatureList<
                    ApplicationFeatureRef<ConsumerSchema, PlanarSummaryFeature>,
                    ApplicationFeatureList<
                        ApplicationFeatureRef<ConsumerSchema, PlanarAlternateSummaryFeature>,
                        ApplicationFeatureList<
                            ApplicationFeatureRef<ConsumerSchema, ParameterFeature>,
                            ApplicationFeatureList<
                                ApplicationFeatureInstanceRef<
                                    ConsumerSchema,
                                    SecondaryPlanarRoot,
                                    PlanarSourceFeature,
                                >,
                                ApplicationFeatureList<
                                    ApplicationFeatureInstanceRef<
                                        ConsumerSchema,
                                        SecondaryPlanarRoot,
                                        PlanarOutputFeature,
                                    >,
                                    ApplicationFeatureList<
                                        ApplicationFeatureInstanceRef<
                                            ConsumerSchema,
                                            DiscoveredPlanarRoot,
                                            PlanarSourceFeature,
                                        >,
                                        ApplicationFeatureList<
                                            ApplicationFeatureInstanceRef<
                                                ConsumerSchema,
                                                DiscoveredPlanarRoot,
                                                PlanarOutputFeature,
                                            >,
                                            ApplicationFeatureList<
                                                ApplicationFeatureInstanceRef<
                                                    ConsumerSchema,
                                                    DiscoveredPlanarRoot,
                                                    PlanarFinalOutputFeature,
                                                >,
                                                ApplicationFeatureList<
                                                    ApplicationFeatureInstanceRef<
                                                        ConsumerSchema,
                                                        RequiredSharedPlanarRoot,
                                                        PlanarSourceFeature,
                                                    >,
                                                    ApplicationFeatureList<
                                                        ApplicationFeatureInstanceRef<
                                                            ConsumerSchema,
                                                            RequiredSharedPlanarRoot,
                                                            PlanarOutputFeature,
                                                        >,
                                                        ApplicationFeatureList<
                                                            ApplicationFeatureInstanceRef<
                                                                ConsumerSchema,
                                                                RequiredSharedPlanarRoot,
                                                                PlanarFinalOutputFeature,
                                                            >,
                                                            ApplicationFeatureLeaf,
                                                        >,
                                                    >,
                                                >,
                                            >,
                                        >,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
>;
