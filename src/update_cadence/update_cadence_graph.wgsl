//! Modified version of the dev tools frame time graph

#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<storage> dts: array<f32>;
@group(1) @binding(1) var<storage> counts: array<u32>;

struct Config {
    dt_min: f32,
    dt_max: f32,
    count_max: u32,
    proportional_width: u32,
}
@group(1) @binding(2) var<uniform> config: Config;

const RED: vec4<f32> = vec4(1.0, 0.0, 0.0, 1.0);
const GREEN: vec4<f32> = vec4(0.0, 1.0, 0.0, 1.0);

// Gets a color based on the update count
fn color_from_count(count: u32) -> vec4<f32> {
    return mix(GREEN, RED, (f32(count)) / f32(config.count_max));
}

// Draw an SDF rectangle
fn sdf_rectangle(pos: vec2<f32>, half_size: vec2<f32>, offset: vec2<f32>) -> f32 {
    let p = pos - offset;
    let dist = abs(p) - half_size;
    let outside_dist = length(max(dist, vec2<f32>(0.0, 0.0)));
    let inside_dist = min(max(dist.x, dist.y), 0.0);
    return outside_dist + inside_dist;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let dt_min = config.dt_min;
    let dt_max = config.dt_max;
    let count_max = config.count_max;

    // The general algorithm is highly inspired by
    // <https://asawicki.info/news_1758_an_idea_for_visualization_of_frame_times>

    let len = min(arrayLength(&dts), arrayLength(&counts));
    var graph_width = 0.0;
    for (var i = 0u; i <= len; i += 1u) {
        let dt = dts[len - i];
        let count = counts[len - i];

        var frame_width: f32;
        if config.proportional_width == 1u {
            frame_width = (dt / dt_min) / f32(len);
        } else {
            frame_width = 0.015;
        }

        let frame_height_factor = f32(count) / f32(count_max);
        // let frame_height_factor = log2(f32(count)) / f32(count_max);
        let frame_height_factor_norm = min(max(0.0, frame_height_factor), 1.0);
        // let frame_height = mix(0.0, 1.0, frame_height_factor_norm);
        let frame_height = frame_height_factor_norm;

        let size = vec2(frame_width, frame_height) / 2.0;
        let offset = vec2(1.0 - graph_width - size.x, 1. - size.y);
        if (sdf_rectangle(in.uv, size, offset) < 0.0) {
            return color_from_count(count);
        }

        graph_width += frame_width;
    }

    return vec4(0.0, 0.0, 0.0, 0.5);
}

