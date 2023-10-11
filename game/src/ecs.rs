//! Useful wrappers around [common::hecs::World]
//!
use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use common::{
    anyhow,
    hecs::{self, EntityBuilder, EntityRef, World},
    serde::{self, de::DeserializeOwned, Deserialize, Serialize},
    serde_json,
};
use components::{
    Building, BuildingGhost, Collider, CombatState, ConstructionSite, Dave, GLTFAsset, Health,
    House, HumanNeeds, Info, Inventory, Job, MaterialOverrides, Parent, PlaceOfWork, Resource,
    RestState, Selected, Storage, TargetIndicator, Targeted, Task, Transform, Velocity, Viking,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct TestComponent {
    foo: usize,
}

#[derive(Copy, Clone)]
pub struct ComponentImpl<T> {
    name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> SerializableComponent for ComponentImpl<T>
where
    T: hecs::Component + Serialize + DeserializeOwned,
{
    fn serialise(
        &self,
        entity: &EntityRef,
        map: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), serde_json::Error> {
        let Some(component) = entity.get::<&T>() else { return Ok(())};
        map.insert(self.name().to_string(), serde_json::to_value(&*component)?);
        Ok(())
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn deserialise(
        &self,
        value: &serde_json::Value,
        entity_builder: &mut EntityBuilder,
    ) -> Result<(), anyhow::Error> {
        let component_data: T = serde_json::from_value(value.clone())?;
        entity_builder.add::<T>(component_data);
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn SerializableComponent> {
        Box::new(Self {
            name: self.name.clone(),
            _phantom: self._phantom.clone(),
        })
    }
}

pub trait SerializableComponent {
    fn name(&self) -> &'static str;
    fn serialise(
        &self,
        entity: &EntityRef,
        map: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), serde_json::Error>;
    fn deserialise(
        &self,
        value: &serde_json::Value,
        entity_builder: &mut EntityBuilder,
    ) -> Result<(), anyhow::Error>;
    fn box_clone(&self) -> Box<dyn SerializableComponent>;
}

/// Cheeky shorthand for creating a Serialiser for some component `T`
pub fn c<T>(name: &'static str) -> (TypeId, Box<dyn SerializableComponent>)
where
    T: hecs::Component + Serialize + DeserializeOwned,
{
    (
        TypeId::of::<T>(),
        Box::new(ComponentImpl {
            name,
            _phantom: PhantomData::<T>,
        }),
    )
}

pub struct SerialisationContext {
    serialisers_by_type_id: HashMap<TypeId, Box<dyn SerializableComponent>>,
    serialisers_by_name: HashMap<String, Box<dyn SerializableComponent>>,
}

impl Default for SerialisationContext {
    fn default() -> Self {
        let serializers = [
            c::<TestComponent>("TestComponent"),
            c::<Dave>("Dave"),
            c::<GLTFAsset>("GLTFAsset"),
            c::<Targeted>("Targeted"),
            c::<TargetIndicator>("TargetIndicator"),
            c::<Collider>("Collider"),
            c::<Parent>("Parent"),
            c::<Velocity>("Velocity"),
            c::<Resource>("Resource"),
            c::<Info>("Info"),
            c::<Selected>("Selected"),
            c::<Task>("Task"),
            c::<Building>("Building"),
            c::<PlaceOfWork>("PlaceOfWork"),
            c::<BuildingGhost>("BuildingGhost"),
            c::<ConstructionSite>("ConstructionSite"),
            c::<Storage>("Storage"),
            c::<Inventory>("Inventory"),
            c::<MaterialOverrides>("MaterialOverrides"),
            c::<Health>("Health"),
            c::<House>("House"),
            c::<HumanNeeds>("HumanNeeds"),
            c::<RestState>("RestState"),
            c::<Transform>("Transform"),
            c::<CombatState>("CombatState"), // TODO
            c::<Job>("Job"),
            c::<Viking>("Viking"),
        ];

        let mut serialisers_by_name = HashMap::new();
        let mut serialisers_by_type_id = HashMap::new();

        for (type_id, serialiser) in &serializers {
            serialisers_by_name.insert(serialiser.name().into(), serialiser.box_clone());
            serialisers_by_type_id.insert(type_id.clone(), serialiser.box_clone());
        }

        Self {
            serialisers_by_type_id,
            serialisers_by_name,
        }
    }
}

impl SerialisationContext {
    pub fn serialise_world(
        &self,
        world: &mut hecs::World,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let mut map = serde_json::Map::new();

        for entity in world.iter() {
            let mut component_map = serde_json::Map::new();
            for type_id in entity.component_types() {
                let Some(serializer) = self.serialisers_by_type_id.get(&type_id) else { continue };
                serializer.serialise(&entity, &mut component_map)?;
            }

            map.insert(entity.entity().to_bits().to_string(), component_map.into());
        }

        Ok(map.into())
    }

    pub fn deserialise_world(
        &self,
        serialised: &serde_json::Value,
    ) -> Result<World, anyhow::Error> {
        let mut world = World::new();

        let map = serialised
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid JSON: {}", serialised.to_string()))?;

        for (entity_id, components) in map {
            let entity = hecs::Entity::from_bits(entity_id.parse()?)
                .ok_or_else(|| anyhow::anyhow!("Invalid bit pattern: {entity_id}"))?;
            let mut entity_builder = hecs::EntityBuilder::new();

            let component_map = components
                .as_object()
                .ok_or_else(|| anyhow::format_err!("Invalid JSON: {}", components.to_string()))?;

            for (component_name, component_value) in component_map {
                let Some(serialiser) = self.serialisers_by_name.get(component_name) else { continue };
                serialiser.deserialise(component_value, &mut entity_builder)?;
            }

            world.spawn_at(entity, entity_builder.build());
        }

        Ok(world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_trivial_serialisation_roundtrip() {
        let mut world = World::new();

        // Spawn an entity with a trivial component
        world.spawn((TestComponent { foo: 42 },));

        // Initialise the serialisation context
        let serialisation_context = SerialisationContext::default();

        // Serialise the world to a `serde_json::Value`
        let serialised_world = serialisation_context.serialise_world(&mut world).unwrap();

        // Deserialise the world from a `serde_json::Value`
        let mut deserialised_world = serialisation_context
            .deserialise_world(&serialised_world)
            .unwrap();

        // Attempt to retrive our trivial component.
        let foo = deserialised_world
            .query_mut::<&TestComponent>()
            .into_iter()
            .next()
            .unwrap()
            .1
            .foo;

        // Ensure it has the correct data
        assert_eq!(foo, 42);
    }

    #[test]
    pub fn test_entity_references_roundtrip() {
        let mut world = World::new();

        // Spawn entity A with a trivial component
        let a = world.spawn((TestComponent { foo: 42 },));

        // Spawn another entity that has a component referencing entity A
        world.spawn((
            TestComponent { foo: 69 },
            Parent::new(a, Default::default()),
        ));

        // Attempt serialisation roundtrip
        let serialisation_context = SerialisationContext::default();
        let serialised_world = serialisation_context.serialise_world(&mut world).unwrap();
        let mut deserialised_world = serialisation_context
            .deserialise_world(&serialised_world)
            .unwrap();

        // Attempt to retrieve our component with a reference
        let a = deserialised_world
            .query_mut::<&Parent>()
            .into_iter()
            .next()
            .unwrap()
            .1
            .entity;

        // Verify this entity has the correct data
        let foo = world.get::<&TestComponent>(a).unwrap().foo;
        assert_eq!(foo, 42);
    }
}
