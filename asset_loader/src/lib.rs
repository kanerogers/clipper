use common::{
    anyhow::{self, format_err as err, Context},
    glam::{UVec2, Vec2, Vec3, Vec4},
    hecs, log,
};
use components::GLTFAsset;
use gltf::Glb;
use image::codecs::png::PngDecoder;
use itertools::izip;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub position: Vec4,
    pub normal: Vec4,
    pub uv: Vec2,
}

impl Vertex {
    pub fn new<T: Into<Vec4>, U: Into<Vec2>>(position: T, normal: T, uv: U) -> Self {
        Self {
            position: position.into(),
            normal: normal.into(),
            uv: uv.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GLTFModel {
    pub primitives: Vec<Primitive>,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub base_colour_texture: Option<Texture>,
    pub base_colour_factor: Vec4,
    pub normal_texture: Option<Texture>,
    pub metallic_roughness_ao_texture: Option<Texture>,
    pub emissive_texture: Option<Texture>,
}

impl Material {
    fn import(primitive: &gltf::Primitive<'_>, blob: &[u8]) -> anyhow::Result<Self> {
        let material = primitive.material();
        let normal_texture = Texture::import(material.normal_texture(), blob).ok();
        let pbr = material.pbr_metallic_roughness();
        let base_colour_factor = pbr.base_color_factor().into();
        let base_colour_texture = Texture::import(pbr.base_color_texture(), blob).ok();
        let metallic_roughness_ao_texture =
            Texture::import(pbr.metallic_roughness_texture(), blob).ok();
        let emissive_texture = Texture::import(material.emissive_texture(), blob).ok();

        Ok(Self {
            base_colour_texture,
            base_colour_factor,
            normal_texture,
            metallic_roughness_ao_texture,
            emissive_texture,
        })
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_colour_texture: Default::default(),
            base_colour_factor: Vec4::ONE,
            normal_texture: Default::default(),
            metallic_roughness_ao_texture: Default::default(),
            emissive_texture: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    /// x, y
    pub dimensions: UVec2,
    /// data is assumed to be R8G8B8A8
    pub data: Vec<u8>,
}

impl Texture {
    fn import<'a, T>(normal_texture: Option<T>, blob: &[u8]) -> anyhow::Result<Self>
    where
        T: AsRef<gltf::Texture<'a>>,
    {
        let texture = normal_texture
            .as_ref()
            .ok_or_else(|| err!("Texture does not exist"))?
            .as_ref();

        let view = match texture.source().source() {
            gltf::image::Source::View {
                view,
                mime_type: "image/png",
            } => Ok(view),
            gltf::image::Source::View { mime_type, .. } => {
                Err(err!("Invalid mime_type {mime_type}"))
            }
            gltf::image::Source::Uri { .. } => {
                Err(err!("Importing images by URI is not supported"))
            }
        }?;

        let image_bytes = &blob[view.offset()..view.length()];
        let decoder = PngDecoder::new(image_bytes)?;
        let image = image::DynamicImage::from_decoder(decoder)?;
        let image = image
            .as_rgba8()
            .ok_or_else(|| err!("Error decoding image"))?;

        Ok(Texture {
            dimensions: image.dimensions().into(),
            data: image.to_vec(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}

pub enum AssetLoadState {
    Loading,
    Failed(String),
    Loaded(GLTFModel),
}

#[derive(Debug)]
pub struct AssetLoader {
    threadpool: futures_executor::ThreadPool,
    jobs: thunderdome::Arena<AssetLoadJob>,
}

type AssetResult = anyhow::Result<GLTFModel>;

pub struct AssetLoadToken {
    _inner: thunderdome::Index,
}

#[derive(Debug)]
struct AssetLoadJob {
    _inner: Receiver<AssetResult>,
}

impl AssetLoadJob {
    pub fn check(&self) -> AssetLoadState {
        match self._inner.try_recv() {
            Ok(asset_result) => match asset_result {
                Ok(a) => AssetLoadState::Loaded(a),
                Err(e) => AssetLoadState::Failed(format!("{e:?}")),
            },
            Err(TryRecvError::Empty) => AssetLoadState::Loading,
            Err(TryRecvError::Disconnected) => AssetLoadState::Failed(
                "The channel was disconnected. You probably loaded this asset already!".into(),
            ),
        }
    }
}

impl AssetLoader {
    pub fn load_assets(&mut self, world: &mut hecs::World) {
        let mut command_buffer = hecs::CommandBuffer::new();

        // Check if there are any assets that are not yet imported
        for (entity, asset_to_import) in world
            .query::<&GLTFAsset>()
            .without::<hecs::Or<&GLTFModel, &AssetLoadToken>>()
            .iter()
        {
            let token = self.load(&asset_to_import.name);
            command_buffer.insert_one(entity, token);
        }

        // Check on the status of any tokens
        for (entity, token) in world.query::<&AssetLoadToken>().iter() {
            match self.check(token) {
                AssetLoadState::Loading => continue,
                AssetLoadState::Failed(e) => {
                    log::error!("Asset failed to load: {e:?}");
                    command_buffer.remove::<(AssetLoadToken, GLTFAsset)>(entity);
                }
                AssetLoadState::Loaded(asset) => {
                    log::info!("Successfully imported asset!");
                    command_buffer.remove_one::<AssetLoadToken>(entity);
                    command_buffer.insert_one(entity, asset);
                }
            }
        }

        command_buffer.run_on(world);
    }

    pub fn new() -> Self {
        let threadpool = futures_executor::ThreadPool::new().unwrap();
        Self {
            threadpool,
            jobs: Default::default(),
        }
    }

    fn check(&self, token: &AssetLoadToken) -> AssetLoadState {
        self.jobs.get(token._inner).unwrap().check()
    }

    fn load<S: Into<String>>(&mut self, asset_name: S) -> AssetLoadToken {
        // oneshot channel
        let (sender, receiver) = std::sync::mpsc::sync_channel(0);
        self.threadpool
            .spawn_ok(load_and_insert(asset_name.into(), sender));
        let index = self.jobs.insert(AssetLoadJob { _inner: receiver });
        AssetLoadToken { _inner: index }
    }
}

async fn load_and_insert(asset_name: String, sender: SyncSender<AssetResult>) {
    let asset_result = load(asset_name);
    sender
        .send(asset_result)
        .unwrap_or_else(|e| log::error!("Failed to send asset: {e:?}"));
}

fn load(asset_name: String) -> anyhow::Result<GLTFModel> {
    let asset_path = format!("{}/../assets/{}", env!("CARGO_MANIFEST_DIR"), asset_name);
    let file = std::fs::read(&asset_path).context(asset_path)?;
    let glb = Glb::from_slice(&file)?;
    let root = gltf::json::Root::from_slice(&glb.json)?;
    let document = gltf::Document::from_json(root)?;
    let blob = glb.bin.ok_or_else(|| err!("No binary found in glTF"))?;
    let node = document
        .nodes()
        .next()
        .ok_or_else(|| err!("No nodes found in glTF"))?;

    let mut primitives = Vec::new();

    for primitive in node
        .mesh()
        .ok_or_else(|| err!("Node has no mesh"))?
        .primitives()
    {
        let vertices = import_vertices(&primitive, &blob)?;
        let indices = import_indices(&primitive, &blob)?;

        let material = Material::import(&primitive, &blob)?;

        primitives.push(Primitive {
            vertices,
            indices,
            material,
        });
    }

    return Ok(GLTFModel { primitives });
}

fn import_vertices(primitive: &gltf::Primitive<'_>, blob: &[u8]) -> anyhow::Result<Vec<Vertex>> {
    let reader = primitive.reader(|_| Some(blob));
    let position_reader = reader
        .read_positions()
        .ok_or_else(|| err!("Primitive has no positions"))?;
    let normal_reader = reader
        .read_normals()
        .ok_or_else(|| err!("Primitive has no normals"))?;
    let uv_reader = reader
        .read_tex_coords(0)
        .ok_or_else(|| err!("Primitive has no UVs"))?
        .into_f32();
    let vertices = izip!(position_reader, normal_reader, uv_reader)
        .map(|(position, normal, uv)| Vertex {
            position: Vec3::from(position).extend(1.),
            normal: Vec3::from(normal).extend(1.),
            uv: uv.into(),
        })
        .collect();
    Ok(vertices)
}

fn import_indices(primitive: &gltf::Primitive<'_>, blob: &[u8]) -> anyhow::Result<Vec<u32>> {
    let reader = primitive.reader(|_| Some(blob));
    let indices = reader
        .read_indices()
        .ok_or_else(|| err!("Primitive has no indices"))?
        .into_u32()
        .collect();
    Ok(indices)
}

#[cfg(test)]
mod tests {
    use components::GLTFAsset;

    use super::*;

    #[test]
    fn loading_assets() {
        env_logger::init();
        let mut asset_loader = AssetLoader::new();
        let mut world = hecs::World::new();
        let entities_to_spawn = 16;
        for i in 0..entities_to_spawn {
            world.spawn((i, GLTFAsset::new("viking_1.glb")));
        }

        loop {
            asset_loader.load_assets(&mut world);

            if world
                .query_mut::<()>()
                .with::<(&GLTFAsset, &GLTFModel)>()
                .without::<&AssetLoadToken>()
                .into_iter()
                .count()
                == 16
            {
                break;
            }
        }

        let (_, model) = world
            .query_mut::<&GLTFModel>()
            .without::<&AssetLoadToken>()
            .into_iter()
            .next()
            .unwrap();

        let primitive = &model.primitives[0];
        assert_eq!(primitive.vertices.len(), 3474);

        let material = &primitive.material;
        assert!(material.base_colour_texture.is_some());
        assert!(material.normal_texture.is_none());
    }
}