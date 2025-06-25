use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;

use crate::tilemap_manager;
use crate::util::KeyboardInput;
use crate::util::IsometricFacing;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
	#[export]
	pub speed: f32,
	pub facing: IsometricFacing,
	pub destination: Option<Vector2>,
	pub tilemap: Option<Gd<TileMapLayer>>,
	pub nav: Option<Gd<AStarGrid2D>>,
	pub state: PlayerMovementState,
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
			speed: 2.5,
			facing: IsometricFacing::SW,
			destination: None,
			tilemap: None,
			nav: None,
			state: PlayerMovementState::Idle,
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
		if !self.has_nav() {
			// If we don't have pathfinding data, request it and wait.
			let gd = self.to_gd();
			let mut sig = self.signals().update_nav();
			sig.emit(&gd);
			return;
		}
	
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
		
		// Input logic.
		if let Some(facing) = KeyboardInput::get_movement() {
			if let PlayerMovementState::Idle = &self.state {
				// Update facing.
				self.facing = facing;
				// Update state.
				self.state = PlayerMovementState::StartMoving;
			}
		}
		
		// State process logic.
		match &self.state {
			PlayerMovementState::Idle => {
				// Play the idle animation.
				sprite.set_animation(&self.facing.get_idle_animation());
				// Reserve the current tile.
				self.reserve_current_tile();
			},
			PlayerMovementState::StartMoving => {
				sprite.set_animation(&self.facing.get_walk_animation());
				if self.try_moving() {
					self.state = PlayerMovementState::Moving;
				} else {
					self.state = PlayerMovementState::Idle;
				}
			},
			PlayerMovementState::Moving => {
				// Play the walking animation.
				sprite.set_animation(&self.facing.get_walk_animation());
				// Keep moving.
				self.keep_moving(delta);
				// If we're done moving, update state to idle.
				if self.destination.is_none() {
					self.state = PlayerMovementState::Idle;
				}
			}
		};
	}
}

impl Player {
	pub fn set_tilemap(&mut self, tilemap: Gd<TileMapLayer>) { self.tilemap = Some(tilemap); }
	pub fn set_nav(&mut self, nav: Gd<AStarGrid2D>) { self.nav = Some(nav); }

	/// Check whether the player has pathfinding data.
	fn has_nav(&self) -> bool {
		self.tilemap.is_some() && self.nav.is_some()
	}
	
	fn reserve_current_tile(&mut self) {
		if !self.has_nav() { return; }
	
		let tilemap = self.tilemap.as_ref().unwrap();
		let pos = self.base().get_position();
		let gridpos = tilemap_manager::global_to_grid(&tilemap, pos);
		
		let mut sig = self.signals().reserve_tile();
		sig.emit(gridpos);
	}
	
	fn unreserve_current_tile(&mut self) {
		if !self.has_nav() { return; } 
		let tilemap = self.tilemap.as_ref().unwrap();
		let pos = self.base().get_position();
		let gridpos = tilemap_manager::global_to_grid(&tilemap, pos);
		
		let mut sig = self.signals().unreserve_tile();
		sig.emit(gridpos);
	}
	
	/// Calculate the destination coordinates for movement. The destination is always 1 tile in the direction you're facing.
	fn calculate_movement(&self) -> Vector2 {
		let position = self.base().get_position();
		let movement_vector =  self.facing.get_movement_vector(32.0);
		let destination = position + movement_vector;
		destination
	}
	
	/// Check for collision in the direction you're currently facing. If you're allowed to move, move.
	fn try_moving(&mut self) -> bool {
		if !self.has_nav() { return false; }
		
		// Calculate where we're going, based on our current facing.
		let destination = self.calculate_movement();
		
		// Convert to grid coordinates.
		let tilemap = self.tilemap.as_ref().unwrap();
		let destination_grid = tilemap_manager::global_to_grid(&tilemap, destination);
		
		// If the destination is occupied, we can't move.
		let nav = self.nav.as_mut().unwrap();
		if nav.is_point_solid(destination_grid) {
			return false;
		}
		
		// Otherwise, reserve the tile for movement...
		let mut sig = self.signals().reserve_tile();
		sig.emit(destination_grid);
		
		// Unreserve our current tile...
		self.unreserve_current_tile();
		
		// And start moving by updating our destination.
		self.destination = Some(destination);
		true
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

pub enum PlayerMovementState {
	Idle,
	StartMoving,
	Moving
}
