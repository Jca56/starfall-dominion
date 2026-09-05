// xy: physical surface dimensions; z: the current compositor scale.
@group(0) @binding(0) var<uniform> viewport: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

fn hash(cell: vec2<f32>, seed: f32) -> vec3<f32> {
    let value = vec3<f32>(
        dot(cell, vec2<f32>(127.1, 311.7)),
        dot(cell, vec2<f32>(269.5, 183.3)),
        dot(cell, vec2<f32>(419.2, 371.9)),
    );
    return fract(sin(value + seed) * 43758.5453);
}

fn stars(point: vec2<f32>, cell_size: f32, seed: f32, radius: f32) -> vec3<f32> {
    let cell = floor(point / cell_size);
    let random = hash(cell, seed);
    if random.z < 0.42 {
        return vec3<f32>(0.0);
    }

    let center = (cell + 0.18 + random.xy * 0.64) * cell_size;
    let distance = length(point - center);
    let size = radius * (0.65 + random.z * 0.55);
    let antialias = 0.65 / viewport.z;
    let core = 1.0 - smoothstep(max(0.0, size - antialias), size + antialias, distance);
    let glow = exp(-distance * distance / (size * size * 9.0)) * 0.13;
    let brightness = 0.22 + random.z * random.z * 0.75;
    let tint = mix(vec3<f32>(0.52, 0.72, 1.0), vec3<f32>(1.0, 0.86, 0.66), random.x);
    return tint * (core + glow) * brightness;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let logical_size = viewport.xy / viewport.z;
    let point = position.xy / viewport.z;
    // Normalize by height so the nebula keeps its shape at any aspect ratio.
    let centered = (point - logical_size * 0.5) / logical_size.y;
    let cloud_point = centered - vec2<f32>(0.24, -0.12);
    let cloud = exp(-dot(cloud_point * vec2<f32>(1.1, 2.9), cloud_point * vec2<f32>(1.1, 2.9)) * 3.0);
    let ribbon = 0.5 + 0.5 * sin(centered.x * 7.0 + centered.y * 4.0);
    let violet_point = centered + vec2<f32>(0.4, -0.2);
    let violet = exp(-dot(violet_point, violet_point) * 8.0);

    var color = vec3<f32>(0.0018, 0.0032, 0.009);
    color += vec3<f32>(0.004, 0.014, 0.028) * cloud * ribbon;
    color += vec3<f32>(0.010, 0.003, 0.016) * violet;
    color += stars(point, 31.0, 17.0, 0.65) * 0.5;
    color += stars(point + vec2<f32>(173.0, 81.0), 67.0, 43.0, 1.15) * 0.8;
    color += stars(point + vec2<f32>(29.0, 237.0), 139.0, 91.0, 1.85);

    return vec4<f32>(color, 1.0);
}
