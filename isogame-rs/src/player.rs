use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::RayCast2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;

use crate::character::Character;
use crate::util::KeyboardInput;
use crate::util::IsometricFacing;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
	#[export]
	speed: f32,
	facing: IsometricFacing,
	destination: Option<Vector2>,
	tilemap: Option<Gd<TileMapLayer>>,
	nav: Option<Gd<AStarGrid2D>>,
	base: Base<CharacterBody2D>
}

#[godot_api]
impl Player {
	#[signal]
	pub fn reserve_tile(coords: Vector2i);
	#[signal]
	pub fn unreserve_tile(coords: Vector2i);
	#[signal]
	pub fn update_nav(instance: Gd<Player>);
}

#[godot_api]
impl ICharacterBody2D for Player {
	fn init(base: Base<CharacterBody2D>) -> Self {
		Self {
			speed: 3.0,
			facing: IsometricFacing::SW,
			destination: None,
			tilemap: None,
			nav: None,
			base
		}
	}
	
	fn ready(&mut self) {
		// Play sprite animation.
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
		sprite.play();
		
		// Add to the entities group.
		self.base_mut().add_to_group("entities");
	}
}

/*

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
	#[export]
	speed: f32,
	facing: IsometricFacing,
	destination: Option<Vector2>,
	base: Base<CharacterBody2D>
}

#[godot_api]
impl Player {
	#[signal]
	pub fn reserve_tile(instance: Gd<Player>);
	#[signal]
	pub fn unreserve_tile(coords: Vector2);
}

#[godot_api]
impl ICharacterBody2D for Player {
	fn init(base: Base<CharacterBody2D>) -> Self {
		Self {
			speed: 3.0,
			facing: IsometricFacing::SW,
			destination: None,
			base
		}
	}
	
	fn ready(&mut self) {
		// Play sprite animation.
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
		sprite.play();
		
		// Add to the entities group.
		self.base_mut().add_to_group("entities");
	}
	
	fn physics_process(&mut self, delta: f64) {
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");

		if self.is_moving() {
			// If we are moving, set walk animation and keep moving.
			sprite.set_animation(&self.facing.get_walk_animation());
			self.keep_moving(delta);
		} else if let Some(facing) = KeyboardInput::get_movement() {
			// If we are not moving, check if a movement key is currently pressed.
			self.facing = facing;
			sprite.set_animation(&self.facing.get_walk_animation());
			self.try_moving();
		} else {
			// Otherwise, play the idle animation.
			sprite.set_animation(&self.facing.get_idle_animation());
		}
	}
}

impl Character for Player {
	fn get_destination(&self) -> Option<Vector2> { self.destination.clone() }
	fn get_position(&self) -> Vector2 { self.base().get_position() }
	fn get_facing(&self) -> IsometricFacing { self.facing.clone() }
	fn get_raycast(&self) -> Gd<RayCast2D> { self.base().get_node_as("RayCast2D") }
	fn get_speed(&self) -> f32 { self.speed }
	
	fn set_destination(&mut self, destination: Option<Vector2>) { self.destination = destination; }
	fn set_position(&mut self, position: Vector2) { self.base_mut().set_position(position);	}
	
	fn emit_reserve_signal(&mut self) {
		let gd = self.to_gd();
		let mut sig = self.signals().reserve_tile();
		sig.emit(&gd);
	}
	
	fn emit_unreserve_signal(&mut self, coords: Vector2) {
		let mut sig = self.signals().unreserve_tile();
		sig.emit(coords);
	}
}

*/
