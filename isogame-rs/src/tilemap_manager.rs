use godot::prelude::*;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::TileMapLayer;

use crate::player::Player;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct TileMapManager {
	tilemap: Option<Gd<TileMapLayer>>,
	reserved_tiles: Vec<Vector2i>,
	base: Base<Node2D>
}

#[godot_api]
impl INode2D for TileMapManager {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			tilemap: None,
			reserved_tiles: Vec::new(),
			base
		}
	}
	
	fn ready(&mut self) {
		let tilemap : Gd<TileMapLayer> = self.base().get_node_as("TerrainLayer");
		self.tilemap = Some(tilemap);
		
		self.lock_entities();
		self.register_tile_signals(); // TODO this is only done once at startup - what if more entities appear???
	}
}

impl TileMapManager {
	fn lock_entities(&self) {
		let tilemap = self.tilemap.as_ref().unwrap();
	
		let mut tree = self.base().get_tree().unwrap();
		let entities = tree.get_nodes_in_group("entities");
		
		// Lock all entities to the isometric grid.
		for node in entities.iter_shared() {
			// First, convert the entity's global coordinates to grid coordinates.
			let mut node2d : Gd<Node2D> = node.cast();
			let pos = node2d.get_position();
			let local_pos = tilemap.to_local(pos);
			let grid_pos = tilemap.local_to_map(local_pos);
			
			// Then convert them back into global coordinates.
			let mut new_pos = tilemap.map_to_local(grid_pos);
			new_pos = tilemap.to_global(new_pos);
			node2d.set_position(new_pos);
		}
	}
	
	fn register_tile_signals(&mut self) {
		let mut tree = self.base().get_tree().unwrap();
		let entities = tree.get_nodes_in_group("entities");
		
		// TODO TODO TODO TEMPORARY ONLY - UNSAFE CAST TO PLAYER!!! TODO TODO TODO
		for node in entities.iter_shared() {
			let player : Gd<Player> = node.cast();
			let reserve = player.signals().reserve_tile();
			let unreserve = player.signals().unreserve_tile();
			
			reserve.connect_other(self, Self::on_reserve_tile);
			unreserve.connect_other(self, Self::on_unreserve_tile);
		}
	}
	
	fn on_reserve_tile(&mut self, mut instance: Gd<Player>) {
		let mut player = instance.bind_mut();
		let coords = player.calculate_destination();
	
		// Check if the tile is currently occupied.
		let tilemap = self.tilemap.as_ref().unwrap();
		let local_pos = tilemap.to_local(coords);
		let grid_pos = tilemap.local_to_map(local_pos);
		
		if self.reserved_tiles.contains(&grid_pos) {
			return;
		}
		
		self.reserved_tiles.push(grid_pos);
		player.start_moving();
	}
	
	fn on_unreserve_tile(&mut self, coords: Vector2) {
		let tilemap = self.tilemap.as_ref().unwrap();
		let local_pos = tilemap.to_local(coords);
		let grid_pos = tilemap.local_to_map(local_pos);
		
		self.reserved_tiles.retain(|i| {
			if *i == grid_pos { return false; }
			true
		});
	}
}
