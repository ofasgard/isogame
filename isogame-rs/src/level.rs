use godot::prelude::*;
use godot::classes::Node;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::Area2D;
use godot::classes::IArea2D;
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
		
		self.register_warp_signals();
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
	
	fn change_level(&mut self, packed_level: Gd<PackedScene>, spawn_point: Vector2) {
		let old_level = self.current_level.as_mut().unwrap();
		old_level.queue_free();

		let new_level = self.load_level(packed_level, spawn_point);
		self.current_level = Some(new_level);
	}
	
	fn register_warp_signals(&mut self) {
		let mut tree = self.base().get_tree().unwrap();
		let nodes = tree.get_nodes_in_group("warps");
		for node in nodes.iter_shared() {
			let warp : Gd<LevelWarp> = node.cast();
			warp.signals().warp_entered().connect_other(self, Self::on_warp_entered);
		}
	}
	
	fn on_warp_entered(&mut self, body: Gd<Node2D>, level: Gd<PackedScene>, coords: Vector2) {
		if body.get_class().to_string().as_str() == "Player" {
			self.change_level(level, coords);
		}
	}
}

#[derive(GodotClass)]
#[class(base=Area2D,init)]
pub struct LevelWarp {
	#[export]
	level: Option<Gd<PackedScene>>,
	
	#[export]
	coords: Vector2,

	base: Base<Area2D>
}

#[godot_api]
impl LevelWarp {
	#[signal]
	fn warp_entered(body: Gd<Node2D>, level: Gd<PackedScene>, coords: Vector2);
}

#[godot_api]
impl IArea2D for LevelWarp {
	fn ready(&mut self) {
		let sig = self.signals().body_entered();
		sig.connect_self(Self::on_body_entered);
	}
}

impl LevelWarp {
	fn on_body_entered(&mut self, body: Gd<Node2D>) {
		let level = &self.level.clone().unwrap();
		let coords = self.coords.clone();
	
		let mut sig = self.signals().warp_entered();
		sig.emit(&body, level, coords);
	}
}
