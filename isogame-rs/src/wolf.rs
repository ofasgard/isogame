use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;
use godot::classes::Area2D;

use crate::tilemap_manager;
use crate::player::Player;
use crate::util::IsometricFacing;
use crate::util::PathfindingResult;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Wolf {
	#[export]
	speed: f32,
	facing: IsometricFacing,
	destination: Option<Vector2>,
	tilemap: Option<Gd<TileMapLayer>>,
	nav: Option<Gd<AStarGrid2D>>,
	base: Base<CharacterBody2D>
}

#[godot_api]
impl Wolf {
	#[signal]
	pub fn reserve_tile(coords: Vector2i);
	#[signal]
	pub fn unreserve_tile(coords: Vector2i);
	#[signal]
	pub fn update_nav(instance: Gd<Wolf>);
}

#[godot_api]
impl ICharacterBody2D for Wolf {
	fn init(base: Base<CharacterBody2D>) -> Self {
		Self {
			speed: 2.0,
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
		if !self.has_nav() {
			// If we don't have pathfinding data, request it and wait.
			let gd = self.to_gd();
			let mut sig = self.signals().update_nav();
			sig.emit(&gd);
			return;
		}
	
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
		
		if self.is_moving() {
			// If we are moving, set walk animation and keep moving.
			sprite.set_animation(&self.facing.get_walk_animation());
			self.keep_moving(delta);
			return;
		} 
		
		// Otherwise, try and find a path and follow it to the nearest target.
		match self.find_path() {
			PathfindingResult::NoPath => {
				// Play the idle animation and reserve the current tile.
				self.reserve_current_tile();
				sprite.set_animation(&self.facing.get_idle_animation());
			},
			PathfindingResult::ReachedTarget(target_tile) => {
				self.reserve_current_tile();
				self.face_tile(target_tile);
				sprite.set_animation(&self.facing.get_idle_animation()); // for now, until we implement attacking
			},
			PathfindingResult::FoundPath(next_tile) => {
				self.face_tile(next_tile);
				sprite.set_animation(&self.facing.get_walk_animation());
				self.try_moving();
			}
		};
	}
}

impl Wolf {
	pub fn set_tilemap(&mut self, tilemap: Gd<TileMapLayer>) { self.tilemap = Some(tilemap); }
	pub fn set_nav(&mut self, nav: Gd<AStarGrid2D>) { self.nav = Some(nav); }
	
	/// Check whether the mob has pathfinding data.
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
	
	/// Check whether the character is currently moving, i.e. whether they have a destination.
	fn is_moving(&self) -> bool {
		self.destination.is_some()
	}
	
	/// Check whether we can find a path to a nearby player. Returns the next tile of the path.
	fn find_path(&mut self) -> PathfindingResult {
		if !self.has_nav() { return PathfindingResult::NoPath; } 
		let tilemap = self.tilemap.as_ref().unwrap();
		
		// Get a list of nearby bodies.
		let search_radius : Gd<Area2D> = self.base().get_node_as("SearchRadius");
		let candidates = search_radius.get_overlapping_bodies();
		
		// Search for a player.
		let mut target : Option<Gd<Player>> = None;
		for candidate in candidates.iter_shared() {
			if candidate.get_class().to_string().as_str() == "Player" {
				let player : Gd<Player> = candidate.cast();
				target = Some(player);
			}
		}
		
		// If we didn't find anyone, give up.
		if target.is_none() { return PathfindingResult::NoPath; }

		// Get the path origin and end.
		let origin_pos = tilemap_manager::global_to_grid(&tilemap, self.base().get_position());
		let target_pos = tilemap_manager::global_to_grid(&tilemap, target.unwrap().get_position());
		
		let tilemap = self.tilemap.as_mut().unwrap();
		
		// Check whether we already reached the target.
		let wolf_neighbours = tilemap.get_surrounding_cells(origin_pos);
		for neighbour in wolf_neighbours.iter_shared() {
			if neighbour == target_pos { return PathfindingResult::ReachedTarget(neighbour); }
		}
		
		// Get the four cells around the target.
		let target_neighbours = tilemap.get_surrounding_cells(target_pos);
		
		let nav = self.nav.as_mut().unwrap();
		for neighbour in target_neighbours.iter_shared() {
			// Perform pathfinding.
			let path = nav.get_id_path(origin_pos, neighbour);
			// Check if a valid path was returned. If so, return index 1, which is the next tile.
			if let Some(tile) = path.get(1) { return PathfindingResult::FoundPath(tile); }
		}
		
		// If no valid paths we returned, give up.
		PathfindingResult::NoPath
	}
	
	/// Update your facing to move into the specified adjacent tile.
	fn face_tile(&mut self, tile: Vector2i) {
		let tilemap = self.tilemap.as_ref().unwrap();
		
		let pos = self.base().get_position();
		let tilepos = tilemap_manager::grid_to_global(&tilemap, tile);
		let movement_vector = tilepos - pos;
		
		match IsometricFacing::from_movement_vector(movement_vector, 32.0) {
			Some(facing) => self.facing = facing,
			None => panic!("Wolf tried to move in an illegal direction!")
		};
	}
	
	/// Check for collision in the direction you're currently facing. If you're allowed to move, move.
	fn try_moving(&mut self) {
		if !self.has_nav() { return; }
		
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
		
		// Otherwise, reserve the tile for movement...
		let mut sig = self.signals().reserve_tile();
		sig.emit(destination_grid);
		
		// Unreserve our current tile...
		self.unreserve_current_tile();
		
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
