use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::RayCast2D;

use crate::util::KeyboardInput;
use crate::util::IsometricFacing;

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

impl Player {
	pub fn is_moving(&self) -> bool {
		match &self.destination {
			Some(_) => true,
			None => false
		}
	}
	
	pub fn calculate_destination(&self) -> Vector2 {
		let position = self.base().get_position();
		let movement_vector =  self.facing.get_movement_vector(32.0);
		let destination = position + movement_vector;
		destination
	}
	
	pub fn try_moving(&mut self) {
		// Determine whether the tile is occupied by something with collision.
		let movement_vector =  self.facing.get_movement_vector(32.0);
		let mut raycast : Gd<RayCast2D> = self.base().get_node_as("RayCast2D");
		raycast.set_target_position(movement_vector);
		raycast.force_raycast_update();
		
		if raycast.is_colliding() {
			return;
		}
		
		// Ask nicely if we're allowed to move.
		let gd = self.to_gd();
		let mut sig = self.signals().reserve_tile();
		sig.emit(&gd);
	}
	
	pub fn start_moving(&mut self) {
		let destination = self.calculate_destination();
		self.destination = Some(destination);	
	}
	
	pub fn keep_moving(&mut self, delta: f64) {
		if let Some(destination) = self.destination {
			// Update our position.
			let mut position = self.base().get_position();
			let velocity = self.facing.get_movement_vector(32.0) * self.speed * (delta as f32);
			position += velocity;
			
			// Check if we have reached our destination.
			if position.distance_to(destination) < 1.0 {
				position = destination;
				
				let mut sig = self.signals().unreserve_tile();
				sig.emit(destination.clone());
				
				self.destination = None;
			}
			
			// Move.
			self.base_mut().set_position(position);
		}
	}
}
