use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::RayCast2D;

use crate::character::Character;
use crate::util::IsometricFacing;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Wolf {
	#[export]
	speed: f32,
	facing: IsometricFacing,
	destination: Option<Vector2>,
	pathfinding_timer: f64,
	path: Array<Vector2i>,
	base: Base<CharacterBody2D>
}

#[godot_api]
impl Wolf {
	#[signal]
	pub fn reserve_tile(instance: Gd<Wolf>);
	#[signal]
	pub fn unreserve_tile(coords: Vector2);
	#[signal]
	pub fn needs_path(instance: Gd<Wolf>);
}

#[godot_api]
impl ICharacterBody2D for Wolf {
	fn init(base: Base<CharacterBody2D>) -> Self {
		Self {
			speed: 3.5,
			facing: IsometricFacing::SW,
			destination: None,
			path: Array::new(),
			pathfinding_timer: 0.0,
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
		} else {
			// Otherwise, play the idle animation and search for a target.
			sprite.set_animation(&self.facing.get_idle_animation());
			self.find_target(delta);
		}
	}
}

impl Wolf {
	pub fn find_target(&mut self, delta: f64) {
		self.pathfinding_timer += delta;

		// If we don't have a path, or if 1 full second has elapsed, update our current path.
		if self.pathfinding_timer >= 1.0 {
			let gd = self.to_gd();
			let mut sig = self.signals().needs_path();
			sig.emit(&gd);
			self.pathfinding_timer = 0.0;
			return;
		}
		
		godot_print!("Path to deer: {:?}", self.path);
		// TODO - find a path and use it to update destination
	}
	
	pub fn set_path(&mut self, path: Array<Vector2i>) { self.path = path; }
}

impl Character for Wolf {
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
