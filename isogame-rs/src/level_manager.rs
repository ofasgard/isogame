use godot::prelude::*;
use godot::classes::Node;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::Area2D;
use godot::classes::IArea2D;
use godot::classes::PackedScene;

use crate::player::Player;
use crate::util::IsometricFacing;

#[derive(GodotClass)]
#[class(base=Node2D,init)]
struct LevelManager {
    #[export]
    level: GString,
    
    #[export]
    player: GString,
    
    #[export]
    player_coords: Vector2,
    
    #[export]
    player_facing: IsometricFacing,
    
    current_level: Option<Gd<Node>>,
    warp: bool,
    base: Base<Node2D>
}

#[godot_api]
impl INode2D for LevelManager {
	fn ready(&mut self) {		
		let packed_level : Gd<PackedScene> = load(&self.level);
		let packed_player : Gd<PackedScene> = load(&self.player);
		
		let level = self.load_level(packed_level, packed_player, self.player_coords, self.player_facing.clone());
		self.current_level = Some(level);
		
		self.register_warp_signals();
	}
	
	fn process(&mut self, _delta: f64) {
		if self.warp {
			let packed_level : Gd<PackedScene> = load(&self.level);
			let packed_player : Gd<PackedScene> = load(&self.player);
			
			self.change_level(packed_level, packed_player, self.player_coords, self.player_facing.clone());
			self.register_warp_signals();
			self.warp = false;
		}
	}
}

impl LevelManager {
	fn load_level(&mut self, packed_level: Gd<PackedScene>, packed_player: Gd<PackedScene>, spawn_point: Vector2, facing: IsometricFacing) -> Gd<Node> {
		// Create the level.
		let mut level : Gd<Node> = packed_level.instantiate().unwrap();
		
		// Create the player.
		let mut player : Gd<Player> = packed_player.instantiate().unwrap().cast();
		player.set_position(spawn_point);
		player.bind_mut().character.facing = facing;
		
		// Add the player to the level.
		level.add_child(&player);
		
		// Add the level to the scene tree.
		self.base_mut().add_child(&level);
		level
	}
	
	fn change_level(&mut self, packed_level: Gd<PackedScene>, packed_player: Gd<PackedScene>, spawn_point: Vector2, facing: IsometricFacing) {
		let old_level = self.current_level.as_mut().unwrap();
		
		// Backup player data.
		let old_player : Gd<Player> = old_level.get_node_as("Player");
		let player_data = old_player.bind().data.clone();
		
		// Delete old level.
		old_level.queue_free();

		// Create new level.
		let new_level = self.load_level(packed_level, packed_player, spawn_point, facing);
		
		// Restore player data.
		let mut new_player : Gd<Player> = new_level.get_node_as("Player");
		new_player.bind_mut().data = player_data;
		
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
	
	fn on_warp_entered(&mut self, body: Gd<Node2D>, level: GString, coords: Vector2, facing: GString) {
		if body.get_class().to_string().as_str() == "Player" {
			self.level = level;
			self.player_coords = coords;
			self.player_facing = IsometricFacing::from_godot(facing);
			self.warp = true;
		}
	}
}

#[derive(GodotClass)]
#[class(base=Area2D,init)]
pub struct LevelWarp {
	#[export]
	level: GString,
	
	#[export]
	coords: Vector2,
	
	#[export]
	facing: IsometricFacing,

	base: Base<Area2D>
}

#[godot_api]
impl LevelWarp {
	#[signal]
	fn warp_entered(body: Gd<Node2D>, level: GString, coords: Vector2, facing: GString);
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
		let level = self.level.clone();
		let coords = self.coords.clone();
		let facing = self.facing.clone();
	
		let mut sig = self.signals().warp_entered();
		sig.emit(&body, &level, coords, &facing.to_godot());
	}
}
