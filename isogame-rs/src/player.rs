use godot::prelude::*;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;

use crate::tilemap_manager;
use crate::character::MovingCharacter;
use crate::util::KeyboardInput;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Player {
	#[export]
	pub speed: f32,
	pub health: f32,
	pub character: MovingCharacter,
	pub movement_state: PlayerMovementState,
	pub animation_state: PlayerAnimationState,
	pub reservation_state: PlayerReservationState,
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
			health: 100.0,
			character: MovingCharacter::new(),
			movement_state: PlayerMovementState::Idle,
			animation_state: PlayerAnimationState::Idle,
			reservation_state: PlayerReservationState::ReserveLocation,
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
		
		// Input logic.
		if let Some(facing) = KeyboardInput::get_movement() {
			if let PlayerMovementState::Idle = &self.movement_state {
				// Update facing.
				self.character.facing = facing;
				// Update state.
				self.movement_state = PlayerMovementState::StartMoving;
				self.animation_state = PlayerAnimationState::Walking;
			}
		}
		
		// Movement logic.
		match &self.movement_state {
			PlayerMovementState::Idle => (),
			PlayerMovementState::StartMoving => {
				let position = self.base().get_position();
				if self.character.try_moving(position) {
					self.movement_state = PlayerMovementState::Moving;
					self.animation_state = PlayerAnimationState::Walking;
					self.reservation_state = PlayerReservationState::ReserveDestination;
				} else {
					self.movement_state = PlayerMovementState::Idle;
					self.animation_state = PlayerAnimationState::Idle;
				}
			},
			PlayerMovementState::Moving => {
				// Keep moving.
				let position = self.base().get_position();
				let new_position = self.character.keep_moving(position, self.speed, delta);
				self.base_mut().set_position(new_position);
				
				if let None = self.character.destination {
					// If we're done moving, change to the idle state.
					self.movement_state = PlayerMovementState::Idle;
					self.animation_state = PlayerAnimationState::Idle;
				}
			}
		};
		
		// Animation logic.
		let mut sprite : Gd<AnimatedSprite2D> = self.base().get_node_as("AnimatedSprite2D");
		
		match &self.animation_state {
			PlayerAnimationState::Idle => sprite.set_animation(&self.character.facing.get_animation("idle")),
			PlayerAnimationState::Walking => sprite.set_animation(&self.character.facing.get_animation("walk"))
		}
		
		// Reservation logic.
		match &self.reservation_state {
			PlayerReservationState::None => (),
			PlayerReservationState::ReserveLocation => {
				self.reserve_current_tile();
				self.reservation_state = PlayerReservationState::None;
			},
			PlayerReservationState::ReserveDestination => {
				self.reserve_facing_tile();
				self.unreserve_current_tile();
				self.reservation_state = PlayerReservationState::None;
			}
		}
	}
}

impl Player {
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
		let gridpos = tilemap_manager::global_to_grid(&tilemap, pos);
		
		let mut sig = self.signals().reserve_tile();
		sig.emit(gridpos);
	}
	
	fn unreserve_current_tile(&mut self) {
		if !self.character.has_nav() { return; } 
		let tilemap = self.character.tilemap.as_ref().unwrap();
		let pos = self.base().get_position();
		let gridpos = tilemap_manager::global_to_grid(&tilemap, pos);
		
		let mut sig = self.signals().unreserve_tile();
		sig.emit(gridpos);
	}
	
	pub fn damage(&mut self, damage: f32) {
		self.health -= damage;
	}
}

pub enum PlayerMovementState {
	Idle,
	StartMoving,
	Moving
}

pub enum PlayerAnimationState {
	Idle,
	Walking
}

pub enum PlayerReservationState {
	None,
	ReserveLocation,
	ReserveDestination
}
