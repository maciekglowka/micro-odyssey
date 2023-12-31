use rogalik::{
    math::vectors::Vector2I,
    storage::{Entity, World}
};
use std::{
    any::{Any, TypeId},
    collections::VecDeque
};

use crate::components::{
    Obstacle, Health, Item, Name, Player, PlayerCharacter,
    Position, Projectile, Paralyzed
};
use crate::events::ActionEvent;
use crate::utils::{are_hostile, get_entities_at_position, spawn_with_position};

pub struct PendingActions(pub VecDeque<Box<dyn Action>>);
pub struct ActorQueue(pub VecDeque<Entity>);

pub type ActionResult = Result<Vec<Box<dyn Action>>, ()>;

pub trait Action {
    fn as_any(&self) -> &dyn Any;
    fn execute(&self, world: &mut World) -> ActionResult;
    fn event(&self) -> ActionEvent { ActionEvent::Other }
    fn score(&self, world: &World) -> i32 { 0 }
    fn type_id(&self) -> TypeId where Self: 'static {
        TypeId::of::<Self>()
    }
}

pub struct Travel {
    pub entity: Entity,
    pub target: Vector2I
}
impl Action for Travel {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let mut position = world.get_component_mut::<Position>(self.entity).ok_or(())?;
        position.0 = self.target;
        Ok(Vec::new())
    }
    fn event(&self) -> ActionEvent {
        ActionEvent::Travel(self.entity, self.target)
    }
    fn score(&self, world: &World) -> i32 {
        let Some(player_position) = world.query::<PlayerCharacter>().with::<Position>()
            .iter()
            .map(|i| i.get::<Position>().unwrap().0)
            .next()
            else { return 0 };

        20 - self.target.manhattan(player_position)
    }
}

pub struct Shoot {
    pub source: Vector2I,
    pub dir: Vector2I,
    pub dist: u32,
    pub damage:  u32
}
impl Shoot {
    fn get_target(&self, world: &World) -> Vector2I {
        let obstacle_positions = world.query::<Obstacle>().with::<Position>()
        .iter()
        .map(|i| i.get::<Position>().unwrap().0)
        .collect::<Vec<_>>();

        // find target - eg. the max dist or first obstacle on the way
        let mut target = self.source;
        for _ in 1..=self.dist {
            target += self.dir;
            if obstacle_positions.contains(&target) { break };
        }
        target
    }
}
impl Action for Shoot {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let target = self.get_target(world);
        let entity = world.spawn_entity();
        let _ = world.insert_component(entity, Projectile{
            damage: self.damage,
            target,
            source: self.source
        });
        Ok(Vec::new())
    }
    fn score(&self, world: &World) -> i32 {
        let Some(player_position) = world.query::<PlayerCharacter>().with::<Position>()
            .iter()
            .map(|i| i.get::<Position>().unwrap().0)
            .next()
            else { return 0 };
        let target = self.get_target(world);
        if target == player_position {
            100
        } else {
            0
        }
    }
}

pub struct Paralyze {
    pub target: Entity,
    pub value: u32
}
impl Action for Paralyze {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let _ = world.insert_component(
            self.target,
            Paralyzed(self.value)
        );
        Ok(Vec::new()) 
    }
}

pub struct Pause {
    pub entity: Entity
}
impl Action for Pause {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        if world.get_component::<PlayerCharacter>(self.entity).is_some() {
            Ok(vec![Box::new(PickItem { entity: self.entity })]) 
        } else {
            Ok(Vec::new())
        }
    }
}


pub struct PlaceBouy {
    pub position: Vector2I,
    pub health:  u32
}
impl Action for PlaceBouy {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let Some(entity) = spawn_with_position(world, "Buoy", self.position)
            else { return Err(()) };
        let _ = world.insert_component(entity, Player);
        let _ = world.insert_component(entity, Health(self.health));
        Ok(Vec::new()) 
    }
    fn score(&self, world: &World) -> i32 {
        // atm whatever ;)
        0
    }
}

pub struct MeleeHit {
    pub entity: Entity,
    pub target: Vector2I,
    pub value: u32
}
impl Action for MeleeHit {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let actions = get_entities_at_position(world, self.target).iter()
            .filter_map(|e| match world.get_component::<Health>(*e) {
                Some(_) => Some(
                    Box::new(Damage { entity: *e, value: self.value }) as Box<dyn Action>
                ),
                None => None
            })
            .collect::<Vec<_>>();
        Ok(actions)
    }
    fn event(&self) -> ActionEvent {
        ActionEvent::Melee(self.entity, self.target, self.value)
    }
    fn score(&self, world: &World) -> i32 {
        if get_entities_at_position(world, self.target).iter().any(
            |e| world.get_component::<Player>(*e).is_some()
        ) {
            200
        } else {
            -50
        }
    }
}

pub struct Damage {
    pub entity: Entity,
    pub value: u32
}
impl Action for Damage {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let mut health = world.get_component_mut::<Health>(self.entity).ok_or(())?;
        health.0 = health.0.saturating_sub(self.value);
        Ok(Vec::new())
    }
    // score is not implemented as it always should be a resulting action
}

pub struct PickItem {
    pub entity: Entity
}
impl Action for PickItem {
    fn as_any(&self) -> &dyn Any { self }
    fn execute(&self, world: &mut World) -> ActionResult {
        let position = world.get_component_mut::<Position>(self.entity).ok_or(())?.0;
        let items = get_entities_at_position(world, position).iter()
            .filter(|e| world.get_component::<Item>(**e).is_some())
            .map(|&e| e)
            .collect::<Vec<_>>();
        
        for item in items {
            world.despawn_entity(item);
        }
        Ok(Vec::new())
    }
    fn score(&self, world: &World) -> i32 {
        // npcs do not pick
        0
    }
}