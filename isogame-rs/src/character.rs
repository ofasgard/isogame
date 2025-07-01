use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;

use crate::level;
use crate::util::IsometricFacing;

pub struct MovingCharacter {
	pub facing: IsometricFacing,
	pub destination: Option<Vector2>,
	pub tilemap: Option<Gd<TileMapLayer>>,
	pub nav: Option<Gd<AStarGrid2D>>,
}

impl MovingCharacter {
	pub fn new() -> Self{
		Self {
			facing: IsometricFacing::SW,
			destination: None,
			tilemap: None,
			nav: None
		}
	}

	pub fn set_tilemap(&mut self, tilemap: Gd<TileMapLayer>) { self.tilemap = Some(tilemap); }
	pub fn set_nav(&mut self, nav: Gd<AStarGrid2D>) { self.nav = Some(nav); }
	
	/// Check whether the character has pathfinding data.
	pub fn has_nav(&self) -> bool {
		self.tilemap.is_some() && self.nav.is_some()
	}
	
	/// Get the character's current location in grid coordinates.
	pub fn get_gridpos(&self, position: Vector2) -> Vector2i {
		let tilemap = self.tilemap.as_ref().unwrap();
		level::global_to_grid(&tilemap, position)
	}
	
	/// Calculate the destination coordinates for movement. The destination is always 1 tile in the direction you're facing.
	pub fn calculate_movement(&self, position: Vector2) -> Vector2 {
		let movement_vector =  self.facing.get_movement_vector(32.0);
		let destination = position + movement_vector;
		destination
	}
	
	/// Get the grid coordinates of the tile in front of you.
	pub fn calculate_movement_grid(&self, position: Vector2) -> Vector2i {
		let tilemap = self.tilemap.as_ref().unwrap();
		let destination = self.calculate_movement(position);
		level::global_to_grid(&tilemap, destination)
	}
	
	/// Check for collision in the direction you're currently facing. If you're allowed to move, move and return true.
	pub fn try_moving(&mut self, position: Vector2) -> bool {
		if !self.has_nav() { return false; }
		
		// Calculate where we're going, based on our current facing.
		let destination = self.calculate_movement(position);
		let destination_grid = self.calculate_movement_grid(position);
		
		// If the destination is occupied, we can't move.
		let nav = self.nav.as_mut().unwrap();
		if nav.is_point_solid(destination_grid) {
			return false;
		}
		
		// And start moving by updating our destination.
		self.destination = Some(destination);
		true
	}
	
	/// Continue moving towards our current destination. Returns the new position.
	pub fn keep_moving(&mut self, mut position: Vector2, speed: f32, delta: f64) -> Vector2 {
		let destination = self.destination.unwrap();
		
		// Update our position.
		let velocity = self.facing.get_movement_vector(32.0) * speed * (delta as f32);
		position += velocity;
		
		// Check if we have reached our destination.
		if position.distance_to(destination) < 1.0 {
			position = destination;
			self.destination = None;
		}
		position
	}
	
	/// Update your facing to move into the specified adjacent tile.
	pub fn face_tile(&mut self, position: Vector2, tile: Vector2i) {
		let tilemap = self.tilemap.as_ref().unwrap();
		
		let tilepos = level::grid_to_global(&tilemap, tile);
		let movement_vector = tilepos - position;
		
		match IsometricFacing::from_movement_vector(movement_vector, 32.0) {
			Some(facing) => self.facing = facing,
			None => panic!("Tried to face a non-adjacent or illegal tile!")
		};
	}
}
