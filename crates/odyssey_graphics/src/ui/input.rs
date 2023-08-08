use odyssey_game::get_ability_actions;

use crate::world_to_tile;
use super::{ButtonState, InputState};

pub fn handle_tile_input(
    world: &rogalik::storage::World,
    state: &InputState
) {
    if state.mouse_button_left != ButtonState::Released { return }

    let query = world.query::<odyssey_game::components::PlayerCharacter>();
    let Some(item) = query.iter().next() else { return };
    let tile = world_to_tile(state.mouse_world_position);

    let Some(actor) = item.get::<odyssey_game::components::Actor>() else { return };
    let Some(mut player) = item.get_mut::<odyssey_game::components::PlayerCharacter>() else { return };
    let ability = actor.abilities[player.active_ability];

    if let Some(cooldown) = ability.cooldown {
        if cooldown.current > 0 { return }
    }

    if let Some(action) = get_ability_actions(item.entity, &ability, world).remove(&tile) {
        player.selected_action = Some(action);
    }
}