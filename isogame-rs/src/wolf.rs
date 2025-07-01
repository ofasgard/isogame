use godot::prelude::*;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::Area2D;

use crate::level;
use crate::character::MovingCharacter;
use crate::player::Player;
use crate::util::PathfindingResult;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Wolf {
	pub speed: f32,
	pub _health: f32,
	pub target: Option<Gd<Player>>,
	pub character: MovingCharacter,
	pub movement_state: WolfMovementState,
	pub animation_state: WolfAnimationState,
	pub reservation_state: WolfReservationState,
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
			speed: 2.75,
			_health: 100.0,
			target: None,
			character: MovingCharacter::new(),
			movement_state: WolfMovementState::Idle,
			animation_state: WolfAnimationState::Idle,
			reservation_state: WolfReservationState::ReserveLocation,
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
		if !self.character.has_nav() {
			// If we don't have pathfinding data, request it and wait.
			self.ask_for_nav();
			return;
		}
		
		// If we are not moving, try and find a path.
		if let WolfMovementState::Idle = &self.movement_state {
			match self.find_path() {
				PathfindingResult::NoPath => (), // If there is no path, do nothing.
				PathfindingResult::ReachedTarget(target_tile) => {
					let position = self.base().get_position();
					self.character.face_tile(position, target_tile);
					
					self.movement_state = WolfMovementState::Bite;
					self.animation_state = WolfAnimationState::Bite;
				},
				PathfindingResult::FoundPath(next_tile) => {
					let position = self.base().get_position();
					self.character.face_tile(position, next_tile);
					
					self.movement_state = WolfMovementState::StartMoving;
					self.animation_state = WolfAnimationState::Walking;
				}
				
			}
		}
		
		// Movement logic.
		match &self.movement_state {
			WolfMovementState::Idle => (),
			WolfMovementState::StartMoving => {
				let position = self.base().get_position();
				if self.character.try_moving(position) {
					self.movement_state = WolfMovementState::Moving;
					self.animation_state = WolfAnimationState::Walking;
					self.reservation_state = WolfReservationState::ReserveDestination;
				} else {
					self.movement_state = WolfMovementState::Idle;
					self.animation_state = WolfAnimationState::Idle;
				}
			},
			WolfMovementState::Moving => {
				// Keep moving.
				let position = self.base().get_position();
				let new_position = self.character.keep_moving(position, self.speed, delta);
				self.base_mut().set_position(new_position);
				
				if let None = self.character.destination {
					// If we're done moving, change to the idle state.
					self.movement_state = WolfMovementState::Idle;
					self.animation_state = WolfAnimationState::Idle;
				}
			},
			WolfMovementState::Bite => {
				let sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
				
				if sprite.get_animation().contains("bite") && !sprite.is_playing() {
					// If the animation hasn't started yet (doesn't contain "bite"), we must wait.
					// If the animation hasn't finished yet (is still playing), we must wait.
					
					let facing_pos = self.character.calculate_movement_grid(self.base().get_position());
					let target = self.target.as_mut().unwrap();
					let target_pos = self.character.get_gridpos(target.get_position());
					
					if target_pos == facing_pos  {
						// If the target hasn't moved, we can damage them.
						target.bind_mut().damage(5.0); // hardcoded bite damage
					}
					
					self.movement_state = WolfMovementState::Idle;
					self.animation_state = WolfAnimationState::Idle;
				}
			}
		};
		
		// Animation logic.
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
		
		match &self.animation_state {
			WolfAnimationState::Idle => {
				sprite.set_animation(&self.character.facing.get_animation("idle"));
				sprite.play();
			},
			WolfAnimationState::Walking => {
				sprite.set_animation(&self.character.facing.get_animation("walk"));
				sprite.play();
			}
			WolfAnimationState::Bite => {
				sprite.set_animation(&self.character.facing.get_animation("bite"));
				sprite.play();
			}
		}
		
		// Reservation logic.
		match &self.reservation_state {
			WolfReservationState::None => (),
			WolfReservationState::ReserveLocation => {
				self.reserve_current_tile();
				self.reservation_state = WolfReservationState::None;
			},
			WolfReservationState::ReserveDestination => {
				self.reserve_facing_tile();
				self.unreserve_current_tile();
				self.reservation_state = WolfReservationState::None;
			}
		}
	}
}

impl Wolf {
	fn ask_for_nav(&mut self) {
		let gd = self.to_gd();
		let mut sig = self.signals().update_nav();
		sig.emit(&gd);
	}

	fn reserve_facing_tile(&mut self) {
		if !self.character.has_nav() { return; }
		
		let gridpos = self.character.calculate_movement_grid(self.base().get_position());
		
		let mut sig = self.signals().reserve_tile();
		sig.emit(gridpos);
	}
	
	fn reserve_current_tile(&mut self) {
		if !self.character.has_nav() { return; }
	
		let tilemap = self.character.tilemap.as_ref().unwrap();
		let pos = self.base().get_position();
		let gridpos = level::global_to_grid(&tilemap, pos);
		
		let mut sig = self.signals().reserve_tile();
		sig.emit(gridpos);
	}
	
	fn unreserve_current_tile(&mut self) {
		if !self.character.has_nav() { return; } 
		let tilemap = self.character.tilemap.as_ref().unwrap();
		let pos = self.base().get_position();
		let gridpos = level::global_to_grid(&tilemap, pos);
		
		let mut sig = self.signals().unreserve_tile();
		sig.emit(gridpos);
	}
	
	/// Check whether we can find a path to a nearby player. Returns the next tile of the path.
	fn find_path(&mut self) -> PathfindingResult {
		if !self.character.has_nav() { return PathfindingResult::NoPath; } 
		let tilemap = self.character.tilemap.as_ref().unwrap();
		
		// Get a list of nearby bodies.
		let search_radius : Gd<Area2D> = self.base().get_node_as("SearchRadius");
		let candidates = search_radius.get_overlapping_bodies();
		
		// Search for a player.
		for candidate in candidates.iter_shared() {
			if candidate.get_class().to_string().as_str() == "Player" {
				let player : Gd<Player> = candidate.cast();
				self.target = Some(player);
			}
		}
		
		// If we didn't find anyone, give up.
		if self.target.is_none() { return PathfindingResult::NoPath; }
		let target = self.target.as_ref().unwrap();

		// Get the path origin and end.
		let origin_pos = level::global_to_grid(&tilemap, self.base().get_position());
		let target_pos = level::global_to_grid(&tilemap, target.get_position());
		
		let tilemap = self.character.tilemap.as_mut().unwrap();
		
		// Check whether we already reached the target.
		let wolf_neighbours = tilemap.get_surrounding_cells(origin_pos);
		for neighbour in wolf_neighbours.iter_shared() {
			if neighbour == target_pos { return PathfindingResult::ReachedTarget(neighbour); }
		}
		
		// Get the four cells around the target.
		let target_neighbours = tilemap.get_surrounding_cells(target_pos);
		
		let nav = self.character.nav.as_mut().unwrap();
		for neighbour in target_neighbours.iter_shared() {
			// Perform pathfinding.
			let path = nav.get_id_path(origin_pos, neighbour);
			// Check if a valid path was returned. If so, return index 1, which is the next tile.
			if let Some(tile) = path.get(1) { return PathfindingResult::FoundPath(tile); }
		}
		
		// If no valid paths we returned, give up.
		PathfindingResult::NoPath
	}
}

pub enum WolfMovementState {
	Idle,
	StartMoving,
	Moving,
	Bite
}

pub enum WolfAnimationState {
	Idle,
	Walking,
	Bite
}

pub enum WolfReservationState {
	None,
	ReserveLocation,
	ReserveDestination
}
