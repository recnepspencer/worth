fn vector_coverage(point: vec2<f32>, bounds: vec4<f32>, kind: u32, width: f32, count: u32) -> f32 {
    let inset = width * 0.5;
    let origin = bounds.xy + vec2<f32>(inset);
    let extent = bounds.zw - bounds.xy - vec2<f32>(width);
    var distance = 1e20;
    var inside = false;
    let first = 7u + u32(data[6].x);
    for (var i = 0u; i < count; i += 1u) {
        let edge = data[first + i];
        let a = origin + edge.xy * extent;
        let b = origin + edge.zw * extent;
        let d = b - a;
        let t = clamp(dot(point - a, d) / max(dot(d,d), 1e-20), 0.0, 1.0);
        distance = min(distance, length(point - a - t*d));
        if (a.y > point.y) != (b.y > point.y) {
            if point.x < (b.x-a.x)*(point.y-a.y)/(b.y-a.y)+a.x { inside = !inside; }
        }
    }
    let signed = select(select(distance, -distance, inside), distance-inset, kind == 2u);
    return clamp(0.5-signed, 0.0, 1.0);
}

fn shadow_coverage(point: vec2<f32>, bounds: vec4<f32>, sigma: f32, radius: f32) -> f32 {
    let margin = 3.0 * sigma;
    let caster = bounds + vec4<f32>(margin,margin,-margin,-margin);


    let distance = max(rounded_distance(point,caster,vec4<f32>(radius)),0.0);
    let tail = exp(-4.5);
    return clamp((exp(-0.5*distance*distance/(sigma*sigma))-tail)/(1.0-tail),0.0,1.0);
}

