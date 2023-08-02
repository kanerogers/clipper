use common::rapier3d::{na, prelude::*};
use common::{glam, hecs, Line};

use crate::Game;
use components::{Info, Transform};

pub struct PhysicsContext {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    debug: DebugRenderPipeline,
}

impl Default for PhysicsContext {
    fn default() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            debug: DebugRenderPipeline::new(
                Default::default(),
                DebugRenderMode::all() & !DebugRenderMode::COLLIDER_AABBS,
            ),
        }
    }
}

impl PhysicsContext {
    pub fn step(&mut self, dt: f32) {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = dt;
        self.physics_pipeline.step(
            &[0., -9.81, 0.].into(),
            &integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    pub fn cast_ray(&self, ray: &Ray) -> Option<hecs::Entity> {
        let Some((handle, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            ray,
            100.,
            true,
            Default::default(),
        ) else { return None };

        println!("Ray hit at {:?}", ray.point_at(toi));

        hecs::Entity::from_bits(self.collider_set.get(handle).unwrap().user_data as _)
    }

    fn render_debug(&mut self, backend: &mut PhysicsRenderer) {
        self.debug.render(
            backend,
            &self.rigid_body_set,
            &self.collider_set,
            &self.impulse_joint_set,
            &self.multibody_joint_set,
            &self.narrow_phase,
        );
    }
}

pub fn physics(game: &mut Game) {
    // create colliders if they're missing
    create_missing_collider_handles(game);

    // udpate rapier colliders; world is authoritative
    update_colliders(game);

    // step
    game.physics_context.step(game.time.delta());

    debug_lines(game);
}

fn debug_lines(game: &mut Game) {
    let mut backend = PhysicsRenderer {
        lines: &mut game.debug_lines,
    };

    game.physics_context.render_debug(&mut backend);
}

struct PhysicsRenderer<'a> {
    lines: &'a mut Vec<Line>,
}

impl<'a> DebugRenderBackend for PhysicsRenderer<'a> {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject,
        a: Point<Real>,
        b: Point<Real>,
        color: [f32; 4],
    ) {
        self.lines.push(Line::new(
            from_na(a),
            from_na(b),
            glam::Vec3::new(color[0], color[1], color[2]),
        ));
    }
}

fn update_colliders(game: &mut Game) {
    for (_, (handle, transform)) in game.world.query::<(&ColliderHandle, &Transform)>().iter() {
        let collider = game.physics_context.collider_set.get_mut(*handle).unwrap();
        collider.set_position(transform.into());
    }
}

fn create_missing_collider_handles(game: &mut Game) {
    let mut command_buffer = hecs::CommandBuffer::new();

    for (_entity, (transform, info)) in game
        .world
        .query::<(&Transform, &Info)>()
        .without::<&ColliderHandle>()
        .iter()
    {
        if info.name == "Ground" {
            continue;
        }
        let _scale = transform.scale;
        // let shape = match geometry {
        //     Geometry::Plane => SharedShape::cuboid(scale.x, scale.y, 0.0),
        //     Geometry::Sphere => SharedShape::ball(scale.x),
        //     Geometry::Cube => SharedShape::cuboid(scale.x, scale.y, scale.z),
        // };

        // let collider = ColliderBuilder::new(shape)
        //     .position(transform.into())
        //     .user_data(entity.to_bits().get() as _)
        //     .active_collision_types(ActiveCollisionTypes::all())
        //     .sensor(true);
        // println!(
        //     "Created collider for {} - {:?}",
        //     info.name, collider.position
        // );

        // let handle = game.physics_context.collider_set.insert(collider.build());

        // command_buffer.insert_one(entity, handle);
    }
    // println!("..done!");

    command_buffer.run_on(&mut game.world);
}

pub fn from_na<T, U>(value: U) -> T
where
    T: FromNa<U>,
{
    T::from_na(value)
}

pub trait FromNa<U> {
    fn from_na(value: U) -> Self;
}

impl FromNa<na::Point3<f32>> for glam::Vec3 {
    fn from_na(value: na::Point3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl FromNa<na::Vector3<f32>> for glam::Vec3 {
    fn from_na(value: na::Vector3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl FromNa<na::Translation3<f32>> for glam::Vec3 {
    fn from_na(value: na::Translation3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl FromNa<na::Quaternion<f32>> for glam::Quat {
    fn from_na(value: na::Quaternion<f32>) -> Self {
        Self::from_xyzw(value.i, value.j, value.k, value.w)
    }
}

impl<T, U> FromNa<na::Unit<T>> for U
where
    U: FromNa<T>,
{
    fn from_na(value: na::Unit<T>) -> Self {
        Self::from_na(value.into_inner())
    }
}
