use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::RayCast2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;

use crate::tilemap_manager;
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
	pub fn set_tilemap(&mut self, tilemap: Gd<TileMapLayer>) { self.tilemap = Some(tilemap); }
	pub fn set_nav(&mut self, nav: Gd<AStarGrid2D>) { self.nav = Some(nav); }
	
	/// Check whether the player has pathfinding data.
	pub fn has_nav(&self) -> bool {
		self.tilemap.is_some() && self.nav.is_some()
	}
	
	/// Calculate the destination coordinates for movement. The destination is always 1 tile in the direction you're facing.
	fn calculate_movement(&self) -> Vector2 {
		let position = self.base().get_position();
		let movement_vector =  self.facing.get_movement_vector(32.0);
		let destination = position + movement_vector;
		destination
	}
	
	/// Check whether the character is currently moving, i.e. whether they have a destination.
	fn is_moving(&self) -> bool {
		self.destination.is_some()
	}
	
	/// Check for collision in the direction you're currently facing. If you're allowed to move, move.
	fn try_moving(&mut self) {
		if !self.has_nav() {
			// If we don't have pathfinding data, request it and wait.
			let gd = self.to_gd();
			let mut sig = self.signals().update_nav();
			sig.emit(&gd);
			return;
		}
		
		// Calculate where we're going, based on our current facing.
		let destination = self.calculate_movement();
		
		// Convert to grid coordinates.
		let tilemap = self.tilemap.as_ref().unwrap();
		let destination_grid = tilemap_manager::global_to_grid(&tilemap, destination);
		
		// If the destination is occupied, we can't move.
		let nav = self.nav.as_mut().unwrap();
		if nav.is_point_solid(destination_grid) {
			return;
		}
		// TODO - sprites and other players aren't on the astargrid2d
		// Previously I used a raycast2d to check for this, but I need them to show up on pathfinding as well
		// could be as simple as reserving the tile during physics_process() and unreserving it when you move?
		// since it's an astargrid2d, you can reserve the same tile repeatedly without causing issues - it's just a call to set_point_solid()
		
		// Otherwise, reserve the tile for movement.
		let mut sig = self.signals().reserve_tile();
		sig.emit(destination_grid);
		
		// And start moving by updating our destination.
		self.destination = Some(destination);
	}
	
	/// Continue moving towards our current destination.
	fn keep_moving(&mut self, delta: f64) {
		let destination = self.destination.unwrap();
		
		// Update our position.
		let mut position = self.base().get_position();
		let velocity = self.facing.get_movement_vector(32.0) * self.speed * (delta as f32);
		position += velocity;
		
		// Check if we have reached our destination.
		if position.distance_to(destination) < 1.0 {
			position = destination;
			
			// Convert destination to grid coordinates and mark the tile as unreserved.
			let tilemap = self.tilemap.as_ref().unwrap();
			let destination_grid = tilemap_manager::global_to_grid(&tilemap, destination);
			
			let mut sig = self.signals().unreserve_tile();
			sig.emit(destination_grid);
			
			self.destination = None;
		}
		
		// Move.
		self.base_mut().set_position(position);
	}
}
