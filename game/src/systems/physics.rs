use common::rapier3d::{self, na, prelude::*};
use common::{hecs, Geometry};

use crate::components::Transform;
use crate::Game;

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
            None,
            &(),
            &(),
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
}

fn update_colliders(game: &mut Game) {
    for (_, (handle, transform)) in game.world.query::<(&ColliderHandle, &Transform)>().iter() {
        let collider = game.physics_context.collider_set.get_mut(*handle).unwrap();
        collider.set_position(transform.into());
    }
}

fn create_missing_collider_handles(game: &mut Game) {
    let mut command_buffer = hecs::CommandBuffer::new();
    for (entity, (geometry, transform)) in game
        .world
        .query::<(&Geometry, &Transform)>()
        .without::<&ColliderHandle>()
        .iter()
    {
        let scale = transform.scale;
        let shape = match geometry {
            Geometry::Plane => SharedShape::cuboid(scale.x, scale.y, 0.001),
            Geometry::Sphere => SharedShape::ball(scale.x),
            Geometry::Cube => SharedShape::cuboid(scale.x, scale.y, scale.z),
        };

        let collider = ColliderBuilder::new(shape)
            .position(transform.into())
            .user_data(entity.to_bits().get() as _)
            .active_collision_types(ActiveCollisionTypes::all())
            .sensor(true);

        let handle = game.physics_context.collider_set.insert(collider.build());

        command_buffer.insert_one(entity, handle);
    }

    command_buffer.run_on(&mut game.world);
}
