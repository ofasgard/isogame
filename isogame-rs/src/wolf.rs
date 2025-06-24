use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;

use crate::util::IsometricFacing;

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
			speed: 3.5,
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
		
		// TODO
	}
}

impl Wolf {
	pub fn set_tilemap(&mut self, tilemap: Gd<TileMapLayer>) { self.tilemap = Some(tilemap); }
	pub fn set_nav(&mut self, nav: Gd<AStarGrid2D>) { self.nav = Some(nav); }
	
	/// Check whether the mob has pathfinding data.
	fn has_nav(&self) -> bool {
		self.tilemap.is_some() && self.nav.is_some()
	}
}
