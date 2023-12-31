use rogalik::math::vectors::Vector2F;
use rogalik::storage::{ComponentSet, Entity, World, WorldEvent};

use odyssey_game::{
    actions::ActorQueue,
    components::{Actor, PlayerCharacter, Position},
    get_ability_actions
};

use crate::SpriteColor;
use crate::globals::TILE_SIZE;

use super::{GraphicsBackend, GraphicsState};

pub fn draw_cursor(
    world: &World,
    backend: &dyn GraphicsBackend
) {
    let Some(queue) = world.get_resource::<ActorQueue>() else { return };
    let query = world.query::<PlayerCharacter>().with::<Position>();
    let Some(item) = query.iter().next() else { return };
    if queue.0.front() != Some(&item.entity) { return };
    let Some(actor) = item.get::<Actor>() else { return };
    let player = item.get::<PlayerCharacter>().unwrap();

    for action in get_ability_actions(item.entity, &actor.abilities[player.active_ability], world) {
        backend.draw_world_sprite(
            "ascii",
            176,
            action.0.as_f32() * TILE_SIZE,
            Vector2F::new(TILE_SIZE, TILE_SIZE),
            SpriteColor(255, 255, 255, 128)
        );
    }

}