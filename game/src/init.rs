use crate::{
    config::{MAX_ENERGY, MAX_HEALTH},
    Game,
};
use common::{hecs, rand, Camera};
use components::{
    Beacon, Collider, Dave, GLTFAsset, Human, Info, Inventory, PlaceOfWork, Resource, Storage,
    Transform, Velocity,
};

pub fn init_game() -> Game {
    let mut camera = Camera::default();
    let mut world = hecs::World::new();

    // dave
    let dave = world.spawn((
        GLTFAsset::new("droid.glb"),
        Dave::new(MAX_ENERGY, MAX_HEALTH),
        Transform::from_position([0., 2., 0.].into()),
        Velocity::default(),
        Info::new("DAVE"),
    ));

    // terrain
    world.spawn((
        Transform::default(),
        Info::new("Ground"),
        GLTFAsset::new("environment.glb"),
    ));

    const STARTING_HUMANS: usize = 10;
    for i in 0..STARTING_HUMANS {
        let x = (rand::random::<f32>() * 50.) - 25.;
        let z = (rand::random::<f32>() * 50.) - 25.;
        world.spawn((
            Collider::default(),
            GLTFAsset::new("viking_1.glb"),
            Human::default(),
            Transform::from_position([x, 1., z].into()),
            Velocity::default(),
            Info::new(format!("Human {i}")),
        ));
    }

    // beacon
    world.spawn((
        Collider::default(),
        GLTFAsset::new("ship.glb"),
        Beacon::default(),
        Transform::default(),
        Info::new("Ship"),
        Inventory::new([]),
        Storage,
    ));

    // mine
    world.spawn((
        Collider::default(),
        GLTFAsset::new("mine.glb"),
        Transform::from_position([30.0, 0.0, 0.0].into()),
        Velocity::default(),
        PlaceOfWork::mine(),
        Inventory::new([(Resource::RawIron, 5000)]),
        Info::new("Mine"),
    ));

    // forge
    world.spawn((
        Collider::default(),
        GLTFAsset::new("forge.glb"),
        Transform::from_position([-30., 0.0, 0.0].into()),
        Velocity::default(),
        PlaceOfWork::forge(),
        Inventory::new([]),
        Info::new("Forge"),
    ));

    // factory
    world.spawn((
        Collider::default(),
        GLTFAsset::new("factory.glb"),
        Transform::from_position([20., 0.0, 30.0].into()),
        Velocity::default(),
        PlaceOfWork::factory(),
        Inventory::new([]),
        Info::new("Factory"),
    ));

    camera.position.y = 3.;
    camera.position.z = 12.;
    camera.distance = 50.;
    camera.desired_distance = camera.distance;
    camera.start_distance = camera.distance;
    Game {
        camera,
        dave,
        world,
        ..Default::default()
    }
}
