use crate::{
    buffer::Buffer,
    descriptors::Descriptors,
    vulkan_context::VulkanContext,
    vulkan_texture::{VulkanTexture, VulkanTextureCreateInfo},
    LineVertex, Vertex, NO_TEXTURE_ID,
};
use common::{glam, Camera, Geometry, Mesh};
use glam::{Vec2, Vec4};
use std::ffi::CStr;

use ash::vk;
use ash::vk::PushConstantRange;
use bytemuck::{Pod, Zeroable};
use thunderdome::Index;
use vk_shader_macros::include_glsl;

const VERTEX_SHADER: &[u32] = include_glsl!("src/shaders/shader.vert");
const FRAGMENT_SHADER: &[u32] = include_glsl!("src/shaders/shader.frag");
const LINE_VERTEX_SHADER: &[u32] = include_glsl!("src/shaders/line.vert");
const LINE_FRAGMENT_SHADER: &[u32] = include_glsl!("src/shaders/line.frag");
pub const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

/// HELLO WOULD YOU LIKE TO RENDER SOME THINGS????
pub struct LazyRenderer {
    /// The render pass used to draw
    render_pass: vk::RenderPass,
    /// One or more framebuffers to draw on. This will match the number of `vk::ImageView` present in [`RenderSurface`]
    framebuffers: Vec<vk::Framebuffer>,
    /// The surface to draw on. Currently only supports present surfaces (ie. the swapchain)
    pub render_surface: RenderSurface,
    /// The pipeline layout used to draw
    mesh_pipeline_layout: vk::PipelineLayout,
    /// The graphics pipeline used to draw meshes
    mesh_pipeline: vk::Pipeline,
    /// The pipeline layout used to draw LINES
    _line_pipeline_layout: vk::PipelineLayout,
    /// The graphics pipeline used to draw lines. It has a funny name.
    line_pipeline: vk::Pipeline,
    /// A single index buffer, shared between all draw calls
    pub index_buffer: Buffer<u32>,
    /// A single vertex buffer, shared between all draw calls
    pub vertex_buffer: Buffer<crate::Vertex>,
    /// A single vertex buffer, shared between all draw calls
    pub line_vertex_buffer: Buffer<crate::LineVertex>,
    /// Textures owned by the user
    user_textures: thunderdome::Arena<VulkanTexture>,
    /// A wrapper around descriptor set functionality
    pub descriptors: Descriptors,
    /// You know. A camera.
    pub camera: Camera,
    /// Some trivial geometry
    pub geometry_offsets: GeometryOffsets,
}

#[derive(Clone)]
/// The surface for lazy-vulkan to draw on. Currently only supports present surfaces (ie. the swapchain)
pub struct RenderSurface {
    /// The resolution of the surface
    pub resolution: vk::Extent2D,
    /// The image format of the surface
    pub format: vk::Format,
    /// The image views to render to. One framebuffer will be created per view
    pub image_views: Vec<vk::ImageView>,
    /// The depth buffers; one per view
    pub depth_buffers: Vec<DepthBuffer>,
}

impl RenderSurface {
    pub fn new(
        vulkan_context: &VulkanContext,
        resolution: vk::Extent2D,
        format: vk::Format,
        image_views: Vec<vk::ImageView>,
    ) -> Self {
        let depth_buffers = create_depth_buffers(vulkan_context, resolution, image_views.len());
        Self {
            resolution,
            format,
            image_views,
            depth_buffers,
        }
    }

    /// Safety:
    ///
    /// After you call this method.. like.. don't use this struct again, basically.
    unsafe fn destroy(&mut self, device: &ash::Device) {
        self.image_views
            .drain(..)
            .for_each(|v| device.destroy_image_view(v, None));
        self.depth_buffers.drain(..).for_each(|d| {
            d.destory(device);
        });
    }
}

impl From<&RenderSurface> for yakui_vulkan::RenderSurface {
    fn from(surface: &RenderSurface) -> Self {
        yakui_vulkan::RenderSurface {
            resolution: surface.resolution,
            format: surface.format,
            image_views: surface.image_views.clone(),
            load_op: vk::AttachmentLoadOp::DONT_CARE,
        }
    }
}

#[derive(Clone)]
pub struct DepthBuffer {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
}
impl DepthBuffer {
    unsafe fn destory(&self, device: &ash::Device) {
        device.destroy_image_view(self.view, None);
        device.destroy_image(self.image, None);
        device.free_memory(self.memory, None);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Push constants!
struct PushConstant {
    mvp: glam::Mat4,
    colour_factor: glam::Vec3,
    texture_id: u32,
}

unsafe impl Zeroable for PushConstant {}
unsafe impl Pod for PushConstant {}

impl PushConstant {
    pub fn new(texture_id: u32, mvp: glam::Mat4, colour_factor: Option<glam::Vec3>) -> Self {
        Self {
            texture_id,
            colour_factor: colour_factor.unwrap_or(glam::Vec3::ONE),
            mvp,
        }
    }
}

impl LazyRenderer {
    /// Create a new [`LazyRenderer`] instance. Currently only supports rendering directly to the swapchain.
    ///
    /// ## Safety
    /// - `vulkan_context` must have valid members
    /// - the members of `render_surface` must have been created with the same [`ash::Device`] as `vulkan_context`.
    pub fn new(vulkan_context: &VulkanContext, render_surface: RenderSurface) -> Self {
        let device = &vulkan_context.device;
        let descriptors = Descriptors::new(vulkan_context);
        let final_layout = vk::ImageLayout::PRESENT_SRC_KHR;

        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: render_surface.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: DEPTH_FORMAT,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        ];

        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            },
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                ..Default::default()
            },
        ];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let render_pass = unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let framebuffers = create_framebuffers(&render_surface, render_pass, device);

        // Populate geometry buffers
        let (initial_indices, initial_vertices, geometry_offsets) = create_initial_geometry();

        let index_buffer = Buffer::new(
            vulkan_context,
            vk::BufferUsageFlags::INDEX_BUFFER,
            &initial_indices,
        );
        let vertex_buffer = Buffer::new(
            vulkan_context,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &initial_vertices,
        );
        let line_vertex_buffer =
            Buffer::new(vulkan_context, vk::BufferUsageFlags::VERTEX_BUFFER, &[]);

        let (mesh_pipeline_layout, mesh_pipeline) =
            create_mesh_pipeline(device, &descriptors, &render_surface, render_pass);
        let (line_pipeline_layout, line_pipeline) =
            create_line_pipeline(device, &render_surface, render_pass);

        Self {
            render_pass,
            descriptors,
            framebuffers,
            render_surface,
            mesh_pipeline_layout,
            mesh_pipeline,
            line_pipeline,
            _line_pipeline_layout: line_pipeline_layout,
            index_buffer,
            vertex_buffer,
            line_vertex_buffer,
            user_textures: Default::default(),
            camera: Default::default(),
            geometry_offsets,
        }
    }

    /// Render the meshes we've been given
    pub fn _render(
        &self,
        vulkan_context: &VulkanContext,
        framebuffer_index: u32,
        meshes: &[Mesh],
        line_vertices: &[LineVertex],
    ) {
        let device = &vulkan_context.device;
        let command_buffer = vulkan_context.draw_command_buffer;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let surface = &self.render_surface;

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[framebuffer_index as usize])
            .render_area(surface.resolution.into())
            .clear_values(&clear_values);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: surface.resolution.width as f32,
            height: surface.resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.mesh_pipeline,
            );
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            let default_scissor = [surface.resolution.into()];

            // We set the scissor first here as it's against the spec not to do so.
            device.cmd_set_scissor(command_buffer, 0, &default_scissor);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer.handle], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.handle,
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.mesh_pipeline_layout,
                0,
                std::slice::from_ref(&self.descriptors.set),
                &[],
            );

            let vp = self.camera.projection * self.camera.matrix();

            for draw_call in meshes {
                let mvp: glam::Mat4 = vp * draw_call.transform;
                device.cmd_push_constants(
                    command_buffer,
                    self.mesh_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&PushConstant::new(
                        draw_call.texture_id,
                        mvp.into(),
                        draw_call.colour,
                    )),
                );

                let IndexBufferEntry {
                    index_count,
                    index_offset,
                    vertex_offset,
                } = self.geometry_offsets.get(draw_call.geometry);

                // Draw the mesh with the indexes we were provided
                device.cmd_draw_indexed(
                    command_buffer,
                    index_count,
                    1,
                    index_offset,
                    vertex_offset as _,
                    1,
                );
            }

            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.line_pipeline,
            );
            device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &[self.line_vertex_buffer.handle],
                &[0],
            );

            // most of these attributes are ignored but.. I'm lazy
            device.cmd_push_constants(
                command_buffer,
                self.mesh_pipeline_layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                bytemuck::bytes_of(&PushConstant::new(NO_TEXTURE_ID, vp, Default::default())),
            );
            device.cmd_draw(command_buffer, (line_vertices.len() * 2) as u32, 1, 0, 1);
            device.cmd_end_render_pass(command_buffer);
        }
    }

    /// Add a "user managed" texture to this [`LazyRenderer`] instance. Returns a [`thunderdome::Index`] that can be used
    /// to refer to the texture.
    ///
    /// ## Safety
    /// - `vulkan_context` must be the same as the one used to create this instance
    pub fn add_user_texture(
        &mut self,
        vulkan_context: &VulkanContext,
        texture_create_info: VulkanTextureCreateInfo<Vec<u8>>,
    ) -> Index {
        let texture =
            VulkanTexture::new(vulkan_context, &mut self.descriptors, texture_create_info);
        self.user_textures.insert(texture)
    }

    /// Clean up all Vulkan related handles on this instance. You'll probably want to call this when the program ends, but
    /// before you've cleaned up your [`ash::Device`], or you'll receive warnings from the Vulkan Validation Layers.
    ///
    /// ## Safety
    /// - After calling this function, this instance will be **unusable**. You **must not** make any further calls on this instance
    ///   or you will have a terrible time.
    /// - `device` must be the same [`ash::Device`] used to create this instance.
    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.device_wait_idle().unwrap();
        self.descriptors.cleanup(device);
        for (_, texture) in &self.user_textures {
            texture.cleanup(device);
        }
        device.destroy_pipeline_layout(self.mesh_pipeline_layout, None);
        device.destroy_pipeline_layout(self.mesh_pipeline_layout, None);
        device.destroy_pipeline(self.mesh_pipeline, None);
        device.destroy_pipeline(self.line_pipeline, None);
        self.index_buffer.cleanup(device);
        self.vertex_buffer.cleanup(device);
        self.destroy_framebuffers(device);
        device.destroy_render_pass(self.render_pass, None);
    }

    /// Update the surface that this [`LazyRenderer`] instance will render to. You'll probably want to call
    /// this if the user resizes the window to avoid writing to an out-of-date swapchain.
    ///
    /// ## Safety
    /// - Care must be taken to ensure that the new [`RenderSurface`] points to images from a correct swapchain
    /// - You must use the same [`ash::Device`] used to create this instance
    pub fn update_surface(&mut self, render_surface: RenderSurface, device: &ash::Device) {
        unsafe {
            self.render_surface.destroy(device);
            self.destroy_framebuffers(device);
        }
        self.framebuffers = create_framebuffers(&render_surface, self.render_pass, device);
        self.render_surface = render_surface;
    }

    unsafe fn destroy_framebuffers(&self, device: &ash::Device) {
        for framebuffer in &self.framebuffers {
            device.destroy_framebuffer(*framebuffer, None);
        }
    }
}

fn create_line_pipeline(
    device: &ash::Device,
    render_surface: &RenderSurface,
    render_pass: vk::RenderPass,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(LINE_VERTEX_SHADER);
    let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(LINE_FRAGMENT_SHADER);

    let vertex_shader_module = unsafe {
        device
            .create_shader_module(&vertex_shader_info, None)
            .expect("Vertex shader module error")
    };

    let fragment_shader_module = unsafe {
        device
            .create_shader_module(&frag_shader_info, None)
            .expect("Fragment shader module error")
    };

    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::builder().push_constant_ranges(&[
                    PushConstantRange {
                        size: std::mem::size_of::<PushConstant>() as _,
                        stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        ..Default::default()
                    },
                ]),
                None,
            )
            .unwrap()
    };

    let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo {
            module: vertex_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            module: fragment_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];
    let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<LineVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];

    let vertex_input_attribute_descriptions = [
        // position
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: bytemuck::offset_of!(LineVertex, position) as _,
        },
        // normals
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: bytemuck::offset_of!(LineVertex, colour) as _,
        },
    ];

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
        .vertex_binding_descriptions(&vertex_input_binding_descriptions);
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::LINE_LIST,
        ..Default::default()
    };
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: render_surface.resolution.width as f32,
        height: render_surface.resolution.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [render_surface.resolution.into()];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        ..Default::default()
    };
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };
    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };
    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: 0,
        depth_write_enable: 0,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        front: noop_stencil_state,
        back: noop_stencil_state,
        max_depth_bounds: 1.0,
        ..Default::default()
    };
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&color_blend_attachment_states);

    let line_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_infos)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_state_info)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass);

    let graphics_pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[line_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline")
    };

    let pipeline = graphics_pipelines[0];
    unsafe {
        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);
    }
    (pipeline_layout, pipeline)
}

fn create_mesh_pipeline(
    device: &ash::Device,
    descriptors: &Descriptors,
    render_surface: &RenderSurface,
    render_pass: vk::RenderPass,
) -> (vk::PipelineLayout, vk::Pipeline) {
    let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(VERTEX_SHADER);
    let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(FRAGMENT_SHADER);

    let vertex_shader_module = unsafe {
        device
            .create_shader_module(&vertex_shader_info, None)
            .expect("Vertex shader module error")
    };

    let fragment_shader_module = unsafe {
        device
            .create_shader_module(&frag_shader_info, None)
            .expect("Fragment shader module error")
    };

    let mesh_pipeline_layout = unsafe {
        device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::builder()
                    .push_constant_ranges(&[PushConstantRange {
                        size: std::mem::size_of::<PushConstant>() as _,
                        stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        ..Default::default()
                    }])
                    .set_layouts(std::slice::from_ref(&descriptors.layout)),
                None,
            )
            .unwrap()
    };

    let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo {
            module: vertex_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            module: fragment_shader_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];
    let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];

    let vertex_input_attribute_descriptions = [
        // position
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: bytemuck::offset_of!(Vertex, position) as _,
        },
        // normals
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: bytemuck::offset_of!(Vertex, normal) as _,
        },
        // UV
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: bytemuck::offset_of!(Vertex, uv) as _,
        },
    ];

    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
        .vertex_binding_descriptions(&vertex_input_binding_descriptions);
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: render_surface.resolution.width as f32,
        height: render_surface.resolution.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }];
    let scissors = [render_surface.resolution.into()];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        ..Default::default()
    };
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };
    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };
    let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        front: noop_stencil_state,
        back: noop_stencil_state,
        max_depth_bounds: 1.0,
        ..Default::default()
    };
    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&color_blend_attachment_states);

    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

    let mesh_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_infos)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_state_info)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state_info)
        .layout(mesh_pipeline_layout)
        .render_pass(render_pass);

    let graphics_pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[mesh_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline")
    };

    let mesh_pipeline = graphics_pipelines[0];
    unsafe {
        device.destroy_shader_module(vertex_shader_module, None);
        device.destroy_shader_module(fragment_shader_module, None);
    }
    (mesh_pipeline_layout, mesh_pipeline)
}

fn create_initial_geometry() -> (Vec<u32>, Vec<Vertex>, GeometryOffsets) {
    let mut vertices = vec![];
    let mut indices = vec![];

    let (plane_vertices, plane_indices) = generate_mesh(Geometry::Plane);
    let plane = IndexBufferEntry::new(plane_indices.len(), indices.len(), vertices.len());
    vertices.extend(plane_vertices);
    indices.extend(plane_indices);

    let (cube_vertices, cube_indices) = generate_mesh(Geometry::Cube);
    let cube = IndexBufferEntry::new(cube_indices.len(), indices.len(), vertices.len());
    vertices.extend(cube_vertices);
    indices.extend(cube_indices);

    let (sphere_vertices, sphere_indices) = generate_mesh(Geometry::Sphere);
    let sphere = IndexBufferEntry::new(sphere_indices.len(), indices.len(), vertices.len());
    vertices.extend(sphere_vertices);
    indices.extend(sphere_indices);

    let offsets = GeometryOffsets {
        plane,
        cube,
        sphere,
    };

    log::debug!("Created geometry offsets: {:?}", offsets);

    (indices, vertices, offsets)
}

fn create_framebuffers(
    render_surface: &RenderSurface,
    render_pass: vk::RenderPass,
    device: &ash::Device,
) -> Vec<vk::Framebuffer> {
    let framebuffers: Vec<vk::Framebuffer> = render_surface
        .image_views
        .iter()
        .zip(&render_surface.depth_buffers)
        .map(|(&present_image_view, depth_buffer)| {
            let framebuffer_attachments = [present_image_view, depth_buffer.view];
            let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&framebuffer_attachments)
                .width(render_surface.resolution.width)
                .height(render_surface.resolution.height)
                .layers(1);

            unsafe {
                device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .unwrap()
            }
        })
        .collect();
    framebuffers
}

fn create_depth_buffers(
    vulkan_context: &VulkanContext,
    resolution: vk::Extent2D,
    len: usize,
) -> Vec<DepthBuffer> {
    (0..len)
        .map(|_| {
            let (image, memory) =
                unsafe { vulkan_context.create_image(&[], resolution, DEPTH_FORMAT) };
            let view = unsafe { vulkan_context.create_image_view(image, DEPTH_FORMAT) };

            DepthBuffer {
                image,
                view,
                memory,
            }
        })
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, Copy)]
pub struct IndexBufferEntry {
    pub index_count: u32,
    pub index_offset: u32,
    pub vertex_offset: u32,
}

impl IndexBufferEntry {
    pub fn new(index_count: usize, index_offset: usize, vertex_offset: usize) -> Self {
        Self {
            index_count: index_count as _,
            index_offset: index_offset as _,
            vertex_offset: vertex_offset as _,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GeometryOffsets {
    plane: IndexBufferEntry,
    cube: IndexBufferEntry,
    sphere: IndexBufferEntry,
}
impl GeometryOffsets {
    fn get(&self, geometry: Geometry) -> IndexBufferEntry {
        match geometry {
            Geometry::Plane => self.plane,
            Geometry::Sphere => self.sphere,
            Geometry::Cube => self.cube,
        }
    }
}

pub fn generate_mesh(geometry: Geometry) -> (Vec<Vertex>, Vec<u32>) {
    match geometry {
        Geometry::Plane => {
            let vertices = vec![
                Vertex {
                    position: Vec4::new(-1.0, -1.0, 0.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, -1.0, 0.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, 0.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, 1.0, 0.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
            ];

            let indices = vec![0, 1, 2, 2, 3, 0];

            (vertices, indices)
        }

        Geometry::Sphere => {
            // Simplified UV Sphere
            let mut vertices = vec![];
            let mut indices = vec![];
            let sectors = 10;
            let stacks = 10;
            let radius = 1.0;
            let pi = std::f32::consts::PI;

            for i in 0..=stacks {
                let stack_angle = pi / 2.0 - i as f32 / stacks as f32 * pi; // starting from pi/2 to -pi/2
                let xy = radius * stack_angle.cos(); // r * cos(u)
                let z = radius * stack_angle.sin(); // r * sin(u)

                for j in 0..=sectors {
                    let sector_angle = j as f32 / sectors as f32 * pi * 2.0; // starting from 0 to 2pi

                    // vertex position (x, y, z)
                    let x = xy * sector_angle.cos(); // r * cos(u) * cos(v)
                    let y = xy * sector_angle.sin(); // r * cos(u) * sin(v)
                    vertices.push(Vertex {
                        position: Vec4::new(x, y, z, 1.0),
                        normal: Vec4::new(x, y, z, 0.0).normalize(), // normalized
                        uv: Vec2::new(j as f32 / sectors as f32, i as f32 / stacks as f32), // normalized
                    });

                    // indices
                    if i != 0 && j != 0 {
                        let a = (sectors + 1) * i + j; // current top right
                        let b = a - 1; // current top left
                        let c = a - (sectors + 1); // previous top right
                        let d = a - (sectors + 1) - 1; // previous top left
                        indices.push(a as u32);
                        indices.push(b as u32);
                        indices.push(c as u32);
                        indices.push(b as u32);
                        indices.push(d as u32);
                        indices.push(c as u32);
                    }
                }
            }

            (vertices, indices)
        }
        Geometry::Cube => {
            let vertices = vec![
                // Front face
                Vertex {
                    position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, 1.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
                // Right face
                Vertex {
                    position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                    normal: Vec4::new(1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                    normal: Vec4::new(1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                    normal: Vec4::new(1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    normal: Vec4::new(1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
                // Back face
                Vertex {
                    position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, -1.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, -1.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, -1.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, 0.0, -1.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
                // Left face
                Vertex {
                    position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                    normal: Vec4::new(-1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                    normal: Vec4::new(-1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                    normal: Vec4::new(-1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                    normal: Vec4::new(-1.0, 0.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
                // Top face
                Vertex {
                    position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, 1.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, 1.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, 1.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, 1.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
                // Bottom face
                Vertex {
                    position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, -1.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                    normal: Vec4::new(0.0, -1.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 0.0),
                },
                Vertex {
                    position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, -1.0, 0.0, 0.0),
                    uv: Vec2::new(1.0, 1.0),
                },
                Vertex {
                    position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                    normal: Vec4::new(0.0, -1.0, 0.0, 0.0),
                    uv: Vec2::new(0.0, 1.0),
                },
            ];

            let indices = vec![
                0, 1, 2, 2, 3, 0, // front
                4, 5, 6, 6, 7, 4, // right
                8, 9, 10, 10, 11, 8, // back
                12, 13, 14, 14, 15, 12, // left
                16, 17, 18, 18, 19, 16, // top
                20, 21, 22, 22, 23, 20, // bottom
            ];

            (vertices, indices)
        }
    }
}
