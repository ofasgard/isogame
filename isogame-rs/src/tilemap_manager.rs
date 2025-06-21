use godot::prelude::*;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::TileMapLayer;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct TileMapManager {
	tilemap: Option<Gd<TileMapLayer>>,
	base: Base<Node2D>
}

#[godot_api]
impl INode2D for TileMapManager {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			tilemap: None,
			base
		}
	}
	
	fn ready(&mut self) {
		let tilemap : Gd<TileMapLayer> = self.base().get_node_as("TerrainLayer");
		self.tilemap = Some(tilemap);
		
		self.lock_entities();
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
	
	fn get_occupied_tiles(&self) -> Vec<Vector2i> {
		let tilemap = self.tilemap.as_ref().unwrap();
		
		let mut tree = self.base().get_tree().unwrap();
		let entities = tree.get_nodes_in_group("entities");
		
		let mut occupied_tiles : Vec<Vector2i> = Vec::new();
		for node in entities.iter_shared() {
			// Convert the entity's global coordinates to grid coordinates.
			let node2d : Gd<Node2D> = node.cast();
			let pos = node2d.get_position();
			let local_pos = tilemap.to_local(pos);
			let grid_pos = tilemap.local_to_map(local_pos);
			occupied_tiles.push(grid_pos);
		}
		occupied_tiles
	}
}
