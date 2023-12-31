use rogalik::storage::{Component, Entity, World};
use rogalik::math::vectors::Vector2I;
use serde::Deserialize;

use crate::actions::Action;
use crate::abilities::Ability;
use crate::items::ItemKind;


#[derive(Clone, Copy, Deserialize)]
pub struct AP {
    pub max: u32,
    #[serde(default)]
    pub current: u32
}

// deserialized components
#[derive(Deserialize)]
pub struct Actor {
    pub abilities: Vec<Ability>,
    pub ap: AP
}
impl Component for Actor {}

#[derive(Deserialize)]
// fixed tile furnishings
pub struct Fixture;
impl Component for Fixture {}

#[derive(Deserialize)]
pub struct Health(pub u32);
impl Component for Health {}

#[derive(Deserialize)]
pub struct Item(pub ItemKind);
impl Component for Item {}

#[derive(Deserialize)]
// actor cannot travel to a blocked tile
pub struct Obstacle;
impl Component for Obstacle {}

#[derive(Deserialize)]
pub struct Tile;
impl Component for Tile {}

#[derive(Deserialize)]
pub struct ViewBlocker;
impl Component for ViewBlocker {}

#[derive(Deserialize)]
pub struct Vortex;
impl Component for Vortex {}


// context-dependet components

pub struct Name (pub String);
impl Component for Name {}

// many can exist in the world
// marks entities 'allied' or spawned by the player
pub struct Player;
impl Component for Player {}

// only one in the game world
// the actual player
pub struct PlayerCharacter {
    pub active_ability: usize,
    pub selected_action: Option<Box<dyn Action>>
}
impl Component for PlayerCharacter {}

pub struct Paralyzed(pub u32);
impl Component for Paralyzed {}

pub struct Projectile {
    pub damage: u32,
    pub source: Vector2I,
    pub target: Vector2I
}
impl Component for Projectile {}

pub struct Position(pub Vector2I);
impl Component for Position {}

pub struct Spawner {
    pub target: String,
    pub countdown: u32
}
impl Component for Spawner {}

pub fn insert_data_components(
    entity: Entity,
    world: &mut World,
    value: &serde_yaml::Value
) {
    let Some(data) = value.as_mapping() else { return };
    for (name, component_data) in data.iter() {
        let Some(name) = name.as_str() else { continue };
        match name {
            "Actor" => insert_single::<Actor>(entity, world, component_data),
            "Fixture" => insert_single::<Fixture>(entity, world, component_data),
            "Health" => insert_single::<Health>(entity, world, component_data),
            "Item" => insert_single::<Item>(entity, world, component_data),
            "Obstacle" => insert_single::<Obstacle>(entity, world, component_data),
            "Tile" => insert_single::<Tile>(entity, world, component_data),
            "ViewBlocker" => insert_single::<ViewBlocker>(entity, world, component_data),
            "Vortex" => insert_single::<Vortex>(entity, world, component_data),
            a => panic!("Unknown component {a}")
        };
    }
}

fn insert_single<T>(
    entity: Entity,
    world: &mut World,
    data: &serde_yaml::Value
) where for<'de> T: 'static + Component + Deserialize<'de> {
    let component = serde_yaml::from_value::<T>(data.clone()).expect(&format!("Could not parse {:?}", data));
    let _ =world.insert_component(entity, component);
}