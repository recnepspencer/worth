use std::{fs, path::Path};

#[derive(Default)]
struct Face {
    id: String,
    path: String,
    face_index: u32,
    byte_length: usize,
    sha256: String,
    fallback_rank: u16,
    emoji: bool,
    last_resort: bool,
}

pub fn generate(manifest: &Path, emoji_test: &Path) -> String {
    let faces = parse_faces(&fs::read_to_string(manifest).expect("read text profile manifest"));
    assert_eq!(faces.len(), 30, "qualified profile face count");
    for (index, face) in faces.iter().enumerate() {
        assert_eq!(usize::from(face.fallback_rank), index, "fallback ranks");
    }
    let face_rows = faces.iter().map(face_source).collect::<String>();
    let embedded_rows = embedded_source(&faces);
    let mut emoji = fs::read_to_string(emoji_test)
        .expect("read emoji-test")
        .lines()
        .filter(|line| line.as_bytes().first().is_some_and(u8::is_ascii_hexdigit))
        .filter_map(|line| {
            let (sequence, disposition) = line.split_once(';')?;
            matches!(
                disposition.split_whitespace().next(),
                Some("fully-qualified" | "component")
            )
            .then(|| sequence_string(sequence))
        })
        .collect::<Vec<_>>();
    emoji.sort();
    emoji.dedup();
    assert_eq!(emoji.len(), 3_953, "qualified RGI sequence count");
    let emoji_rows = emoji
        .into_iter()
        .map(|sequence| format!("    \"{}\",\n", escaped_string(&sequence)))
        .collect::<String>();
    format!(
        "pub(super) const PROFILE_FACES: &[ProfileFaceDescriptor] = &[\n{face_rows}];\n\
         pub(super) fn embedded_profile_inputs() -> Box<[super::UiProfileFontFaceInput]> {{\n{embedded_rows}}}\n\
         pub(super) const UNICODE_17_RGI_EMOJI: &[&str] = &[\n{emoji_rows}];\n"
    )
}

fn embedded_source(faces: &[Face]) -> String {
    let paths = faces
        .iter()
        .map(|face| face.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let arms = paths
        .into_iter()
        .map(|path| {
            format!(
                "            {path:?} => &include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../profiles/worth-ui-global-text-v2/{path}\"))[..],\n"
            )
        })
        .collect::<String>();
    format!(
        "    let mut bytes_by_path = std::collections::BTreeMap::new();\n\
         PROFILE_FACES.iter().map(|face| {{\n\
             let bytes = bytes_by_path.entry(face.path).or_insert_with(|| {{\n\
                 let bytes: &'static [u8] = match face.path {{\n{arms}                    _ => unreachable!(\"qualified profile path\"),\n\
                 }};\n\
                 std::sync::Arc::<[u8]>::from(bytes)\n\
             }}).clone();\n\
             super::UiProfileFontFaceInput {{ id: std::sync::Arc::from(face.id), bytes }}\n\
         }}).collect::<Vec<_>>().into_boxed_slice()\n    "
    )
}

fn parse_faces(source: &str) -> Vec<Face> {
    let mut faces = Vec::new();
    let mut current = None;
    for line in source.lines().map(str::trim) {
        if line == "[[face]]" {
            if let Some(face) = current.take() {
                faces.push(face);
            }
            current = Some(Face::default());
            continue;
        }
        let Some(face) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "id" => face.id = quoted(value),
            "path" => face.path = quoted(value),
            "face_index" => face.face_index = value.parse().expect("face index"),
            "byte_length" => face.byte_length = value.parse().expect("face byte length"),
            "sha256" => face.sha256 = quoted(value),
            "fallback_rank" => face.fallback_rank = value.parse().expect("fallback rank"),
            "emoji" => face.emoji = value == "true",
            "last_resort" => face.last_resort = value == "true",
            _ => {}
        }
    }
    if let Some(face) = current {
        faces.push(face);
    }
    faces
}

fn face_source(face: &Face) -> String {
    let digest = face
        .sha256
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let pair = std::str::from_utf8(pair).expect("digest utf8");
            u8::from_str_radix(pair, 16).expect("digest hex")
        })
        .map(|byte| format!("0x{byte:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "    ProfileFaceDescriptor {{ id: {:?}, path: {:?}, face_index: {}, byte_length: {}, digest: [{digest}], fallback_rank: {}, emoji: {}, last_resort: {} }},\n",
        face.id,
        face.path,
        face.face_index,
        face.byte_length,
        face.fallback_rank,
        face.emoji,
        face.last_resort,
    )
}

fn sequence_string(field: &str) -> String {
    field
        .split_whitespace()
        .map(|value| {
            let scalar = u32::from_str_radix(value, 16).expect("emoji scalar");
            char::from_u32(scalar).expect("emoji Unicode scalar")
        })
        .collect()
}

fn escaped_string(source: &str) -> String {
    source
        .chars()
        .map(|scalar| format!("\\u{{{:X}}}", scalar as u32))
        .collect()
}

fn quoted(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .expect("quoted manifest value")
        .to_owned()
}
