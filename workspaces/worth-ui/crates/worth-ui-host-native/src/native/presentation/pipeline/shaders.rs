pub(super) const RETAINED_TO_SURFACE_SHADER: &str = r#"
@group(0) @binding(0) var retained: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(retained, vec2<i32>(position.xy), 0);
}
"#;

pub(super) const RASTER_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

pub(super) const ANALYTIC_SURFACE_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> data: array<vec4<f32>>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    return output;
}

fn rounded_distance(point: vec2<f32>, bounds: vec4<f32>, radii: vec4<f32>) -> f32 {
    let center = vec2<f32>((bounds.x + bounds.z) * 0.5, (bounds.y + bounds.w) * 0.5);
    let radius = select(
        select(radii.x, radii.w, point.y >= center.y),
        select(radii.y, radii.z, point.y >= center.y),
        point.x >= center.x,
    );
    let half_size = vec2<f32>((bounds.z - bounds.x) * 0.5, (bounds.w - bounds.y) * 0.5);
    let q = abs(point - center) - half_size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

fn coverage(distance: f32) -> f32 {
    return clamp(0.5 - distance, 0.0, 1.0);
}

fn side_enabled(bits: u32, side: u32) -> bool {
    return (bits & (1u << side)) != 0u;
}

fn omitted(side: u32, offset: f32) -> bool {
    let count = u32(data[6].x);
    for (var index = 0u; index < count; index += 1u) {
        let row = data[7u + index];
        if u32(row.x) == side && row.y <= offset && offset < row.z {
            return true;
        }
    }
    return false;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let point = input.position.xy;
    let bounds = data[0];
    let clip = data[1];
    if point.x < clip.x || point.x >= clip.z || point.y < clip.y || point.y >= clip.w {
        discard;
    }
    let radii = data[2];
    let fill = data[3];
    let border = data[4];
    let metrics = data[5];
    let width = metrics.x;
    let opacity = metrics.y;
    let paint_kind = u32(metrics.z);
    let edge_bits = u32(metrics.w);
    let outer = coverage(rounded_distance(point, bounds, radii));
    var inner = outer;
    if width > 0.0 {
        let inner_bounds = bounds + vec4<f32>(width, width, -width, -width);
        inner = coverage(rounded_distance(point, inner_bounds, max(radii - vec4<f32>(width), vec4<f32>(0.0))));
    }
    var border_coverage = max(outer - inner, 0.0);
    let edge_distances = vec4<f32>(
        point.y - bounds.y,
        bounds.z - point.x,
        bounds.w - point.y,
        point.x - bounds.x,
    );
    var side = 0u;
    var edge_distance = edge_distances.x;
    if edge_distances.y < edge_distance { side = 1u; edge_distance = edge_distances.y; }
    if edge_distances.z < edge_distance { side = 2u; edge_distance = edge_distances.z; }
    if edge_distances.w < edge_distance { side = 3u; }
    let offset = select(point.x - bounds.x, point.y - bounds.y, side == 1u || side == 3u);
    let enabled = side_enabled(edge_bits, side) && !omitted(side, offset);
    if !enabled { border_coverage = 0.0; }
    let fill_coverage = select(0.0, outer, paint_kind == 1u || paint_kind == 3u);
    border_coverage = select(0.0, border_coverage, paint_kind == 2u || paint_kind == 3u);
    let border_layer = border * border_coverage;
    let fill_layer = fill * fill_coverage;
    return (border_layer + fill_layer * (1.0 - border_layer.a)) * opacity;
}
"#;

pub(super) const ALPHA_GLYPH_SHADER: &str = r#"
@group(0) @binding(0) var atlas_page: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) texture_uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.texture_uv = texture_uv;
    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas_page, atlas_sampler, input.texture_uv).r;
    let alpha = input.color.a * coverage;
    return vec4<f32>(input.color.rgb * alpha, alpha);
}
"#;

pub(super) const COLOR_GLYPH_SHADER: &str = r#"
@group(0) @binding(0) var atlas_page: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) texture_uv: vec2<f32>,
    @location(2) _foreground: vec4<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.texture_uv = texture_uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(atlas_page, atlas_sampler, input.texture_uv);
}
"#;
