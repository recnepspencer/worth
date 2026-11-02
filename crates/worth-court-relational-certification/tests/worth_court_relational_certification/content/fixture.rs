use std::sync::Arc;
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};
use worth_relational::facade::content::*;
pub(crate) use worth_testkit_relational::content_grid::{
    ExactSampleGrid as Grid, ExactSampleGridCodec as Codec,
};

pub(crate) fn format(name: &str, version: u32, alter: bool) -> RelationalContentFormat {
    let mut formats = RelationalContentFormatRegistry::default();
    let identity = CanonicalDigestId::new([1; 32]);
    formats
        .register(
            identity,
            Arc::new(Codec {
                descriptor: ContentFormat::new(name, version).unwrap(),
                alter,
            }),
        )
        .unwrap();
    formats.get(identity).unwrap().clone()
}
pub(crate) fn grid() -> Grid {
    Grid {
        rows: 2,
        columns: 2,
        samples: vec![i64::MIN, -1, 0, i64::MAX],
    }
}
