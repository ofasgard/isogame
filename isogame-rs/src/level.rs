use godot::prelude::*;
use godot::classes::Node;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::PackedScene;

use crate::player::Player;

#[derive(GodotClass)]
#[class(base=Node2D,init)]
struct LevelScene {
    #[export]
    starting_level: Option<Gd<PackedScene>>,
    
    #[export]
    player: Option<Gd<PackedScene>>,
    
    #[export]
    player_coords: Vector2,
    
    current_level: Option<Gd<Node>>,
    base: Base<Node2D>
}

#[godot_api]
impl INode2D for LevelScene {
	fn ready(&mut self) {		
		let packed_level = self.starting_level.as_mut().unwrap().clone();
		let level = self.load_level(packed_level, self.player_coords);
		self.current_level = Some(level);
	}
}

impl LevelScene {
	fn load_level(&mut self, packed_level: Gd<PackedScene>, spawn_point: Vector2) -> Gd<Node> {
		// Create the level.
		let mut level : Gd<Node> = packed_level.instantiate().unwrap();
		
		// Create the player.
		let packed_player = self.player.as_mut().unwrap();
		let mut player : Gd<Player> = packed_player.instantiate().unwrap().cast();
		player.set_position(spawn_point);
		
		// Add the player to the level.
		level.add_child(&player);
		
		// Add the level to the scene tree.
		self.base_mut().add_child(&level);
		level
	}
}
