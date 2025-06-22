use godot::prelude::*;
use godot::builtin::Vector2;
use godot::classes::CharacterBody2D;
use godot::classes::ICharacterBody2D;
use godot::classes::AnimatedSprite2D;

use crate::character::Character;
use crate::util::IsometricFacing;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Wolf {
	#[export]
	speed: f32,
	facing: IsometricFacing,
	destination: Option<Vector2>,
	base: Base<CharacterBody2D>
}

#[godot_api]
impl Wolf {
	#[signal]
	pub fn reserve_tile(instance: Gd<Wolf>);
	#[signal]
	pub fn unreserve_tile(coords: Vector2);
}

#[godot_api]
impl ICharacterBody2D for Wolf {
	fn init(base: Base<CharacterBody2D>) -> Self {
		Self {
			speed: 3.5,
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
}
