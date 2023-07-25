#include <metal_stdlib>

using namespace metal;

typedef struct {
	float4 position;
	float4 normal;
    float2 uv;
} vertex_t;

struct ColorInOut {
    float4 position [[position]];
    float4 color;
};

typedef struct {
    float4x4 mvp;
    float4 colour;
} Uniforms;

// vertex shader function
vertex ColorInOut triangle_vertex(const device vertex_t* vertex_array [[ buffer(0) ]],
                                  const device Uniforms* uniforms [[ buffer(1) ]],
                                   uint vid [[ vertex_id ]],
                                   uint instance_id [[instance_id]])
{
    ColorInOut out;

    auto device const &v = vertex_array[vid];
    auto device const &u = uniforms[instance_id];
    out.position = u.mvp * v.position;
    out.color = u.colour;

    return out;
}

fragment float4 triangle_fragment(ColorInOut in [[stage_in]])
{
    return in.color;
};
