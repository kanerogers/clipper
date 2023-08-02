#version 450
#define NO_TEXTURE 4294967295
#define WORKFLOW_MAIN 0
#define WORKFLOW_TEXT 1

layout (location = 0) in vec4 in_normal;
layout (location = 1) in vec2 in_uv;
layout (location = 0) out vec4 out_colour;

layout(set = 0, binding = 0) uniform sampler2D textures[1000];

#include "push_constant.glsl"

void main() {
    if (base_colour_texture_id == NO_TEXTURE) {
        out_colour = base_colour_factor;
        return;
    } 
    
    out_colour = base_colour_factor * texture(textures[base_colour_texture_id], in_uv);
}