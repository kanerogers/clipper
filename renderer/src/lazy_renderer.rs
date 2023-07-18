use crate::{
    buffer::Buffer,
    descriptors::Descriptors,
    vulkan_context::VulkanContext,
    vulkan_texture::{VulkanTexture, VulkanTextureCreateInfo},
    LazyVulkanBuilder, Vertex,
};
use common::{glam, Mesh};
use std::ffi::CStr;

use ash::vk;
use ash::vk::PushConstantRange;
use bytemuck::{Pod, Zeroable};
use thunderdome::Index;
use vk_shader_macros::include_glsl;

const VERTEX_SHADER: &[u32] = include_glsl!("src/shaders/shader.vert");
const FRAGMENT_SHADER: &[u32] = include_glsl!("src/shaders/shader.frag");
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
    pipeline_layout: vk::PipelineLayout,
    /// The graphics pipeline used to draw
    graphics_pipeline: vk::Pipeline,
    /// A single index buffer, shared between all draw calls
    pub index_buffer: Buffer<u32>,
    /// A single vertex buffer, shared between all draw calls
    pub vertex_buffer: Buffer<crate::Vertex>,
    /// Textures owned by the user
    user_textures: thunderdome::Arena<VulkanTexture>,
    /// A wrapper around descriptor set functionality
    pub descriptors: Descriptors,
    /// You know. A camera.
    pub camera: Camera,
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

#[derive(Clone, Default)]
pub struct Camera {
    pub position: glam::Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn matrix(&self) -> glam::Affine3A {
        let rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.);
        glam::Affine3A::from_rotation_translation(rotation, self.position).inverse()
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
    pub fn new(
        vulkan_context: &VulkanContext,
        render_surface: RenderSurface,
        builder: &LazyVulkanBuilder,
    ) -> Self {
        let device = &vulkan_context.device;
        let descriptors = Descriptors::new(vulkan_context);
        let final_layout = if builder.with_present {
            vk::ImageLayout::PRESENT_SRC_KHR
        } else {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        };

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

        let index_buffer = Buffer::new(
            vulkan_context,
            vk::BufferUsageFlags::INDEX_BUFFER,
            &builder.initial_indices,
        );
        let vertex_buffer = Buffer::new(
            vulkan_context,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            &builder.initial_vertices,
        );

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

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::builder()
                        .push_constant_ranges(&[PushConstantRange {
                            size: std::mem::size_of::<PushConstant>() as _,
                            stage_flags: vk::ShaderStageFlags::VERTEX
                                | vk::ShaderStageFlags::FRAGMENT,
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
            // UV / texcoords
            vk::VertexInputAttributeDescription {
                location: 1,
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
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass);

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info.build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        };

        let graphics_pipeline = graphics_pipelines[0];

        unsafe {
            device.destroy_shader_module(vertex_shader_module, None);
            device.destroy_shader_module(fragment_shader_module, None);
        }

        Self {
            render_pass,
            descriptors,
            framebuffers,
            render_surface,
            pipeline_layout,
            graphics_pipeline,
            index_buffer,
            vertex_buffer,
            user_textures: Default::default(),
            camera: Default::default(),
        }
    }

    /// Render the meshes we've been given
    pub fn render(&self, vulkan_context: &VulkanContext, framebuffer_index: u32, meshes: &[Mesh]) {
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
                self.graphics_pipeline,
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
                self.pipeline_layout,
                0,
                std::slice::from_ref(&self.descriptors.set),
                &[],
            );

            let aspect_ratio = self.render_surface.resolution.width as f32
                / self.render_surface.resolution.height as f32;
            let mut perspective =
                glam::Mat4::perspective_rh(60_f32.to_radians(), aspect_ratio, 0.01, 1000.);
            perspective.y_axis[1] *= -1.;

            for draw_call in meshes {
                let mvp: glam::Mat4 = perspective * self.camera.matrix() * draw_call.transform;
                device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&PushConstant::new(
                        draw_call.texture_id,
                        mvp.into(),
                        draw_call.colour,
                    )),
                );

                // Draw the mesh with the indexes we were provided
                device.cmd_draw_indexed(
                    command_buffer,
                    draw_call.index_count,
                    1,
                    draw_call.index_offset,
                    0,
                    1,
                );
            }
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
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_pipeline(self.graphics_pipeline, None);
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
