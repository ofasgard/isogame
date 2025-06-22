use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::RayCast2D;

use crate::util::IsometricFacing;

pub trait Character {
	fn get_destination(&self) -> Option<Vector2>;
	fn get_position(&self) -> Vector2;
	fn get_facing(&self) -> IsometricFacing;
	fn get_raycast(&self) -> Gd<RayCast2D>;
	fn get_speed(&self) -> f32;
	fn set_destination(&mut self, destination: Option<Vector2>);
	fn set_position(&mut self, position: Vector2);
	fn emit_reserve_signal(&mut self);
	fn emit_unreserve_signal(&mut self, coords: Vector2);

	/// Check whether the character is currently moving, i.e. whether they have a destination.
	fn is_moving(&self) -> bool {
		match self.get_destination() {
			Some(_) => true,
			None => false
		}
	}
	
	/// Calculate the destination coordinates for movement. The destination is always 1 tile in the direction you're facing.
	fn calculate_destination(&self) -> Vector2 {
		let position = self.get_position();
		let movement_vector =  self.get_facing().get_movement_vector(32.0);
		let destination = position + movement_vector;
		destination
	}
	
	/// Check for collision in the direction you're currently facing. Then, send a movement request to the TileMapManager by emitting the `reserve_tile` signal.
	fn try_moving(&mut self) {
		// Determine whether the tile is occupied by something with collision.
		let movement_vector =  self.get_facing().get_movement_vector(32.0);
		let mut raycast : Gd<RayCast2D> = self.get_raycast();
		raycast.set_target_position(movement_vector);
		raycast.force_raycast_update();
		
		if raycast.is_colliding() {
			return;
		}
		
		// Ask nicely if we're allowed to move.
		self.emit_reserve_signal();
	}
	
	/// Set our destination, which also marks us as "moving".
	fn start_moving(&mut self) {
		let destination = self.calculate_destination();
		self.set_destination(Some(destination));	
	}
	
	/// Continue moving towards our current destination.
	fn keep_moving(&mut self, delta: f64) {
		if let Some(destination) = self.get_destination() {
			// Update our position.
			let mut position = self.get_position();
			let velocity = self.get_facing().get_movement_vector(32.0) * self.get_speed() * (delta as f32);
			position += velocity;
			
			// Check if we have reached our destination.
			if position.distance_to(destination) < 1.0 {
				position = destination;
				
				self.emit_unreserve_signal(destination.clone());
				self.set_destination(None);
			}
			
			// Move.
			self.set_position(position);
		}
	}
}
