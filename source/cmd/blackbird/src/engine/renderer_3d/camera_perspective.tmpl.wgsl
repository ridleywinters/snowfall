[[declaration]]
struct CameraUniform {
    position        : vec4<f32>,
    view_proj       : mat4x4<f32>,
    inv_proj        : mat4x4<f32>,
    inv_view        : mat4x4<f32>,
    _unused0        : f32,
    _unused1        : f32,
    _unused2        : f32,
    _unused3        : f32,
    light_direction : vec3<f32>,
}

[[binding]]
var<uniform>       camera              : CameraUniform;