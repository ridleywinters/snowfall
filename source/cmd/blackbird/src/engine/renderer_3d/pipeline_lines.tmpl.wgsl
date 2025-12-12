//===========================================================================//
// Declarations
//===========================================================================//

{{#each declarations}}
{{this}}
{{/each}}

//===========================================================================//
// Bindings
//===========================================================================//

{{#each bindings}}
@group(0) @binding({{@index}}) {{this}}
{{/each}}


//===========================================================================//
// Vertex Input and Output
//===========================================================================//

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) color : vec3<f32>,
};

struct FragInput {
    @builtin(position) position : vec4<f32>, // Output clip space position for the line
    @location(0) color : vec3<f32>, // Color for the fragment
};

//===========================================================================//
// Vertex Shader
//===========================================================================//

@vertex
fn vs_main(
    vertex : VertexInput,
) -> FragInput {    

    var out : FragInput;
    out.position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    out.color = vertex.color;
    return out;
}

//===========================================================================//
// Fragment Shader
//===========================================================================//

@fragment
fn fs_main(in: FragInput) -> @location(0) vec4<f32> {    
    return vec4<f32>(in.color, 1.0);
}
