layout(push_constant) uniform push_constants {
    uint emissive_texture_id;
    uint metallic_roughness_ao_texture_id;
    uint normal_texture_id;
    uint base_colour_texture_id;
    uint base_colour_factor;
    mat4 mvp;
};