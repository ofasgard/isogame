use godot::prelude::*;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;
use godot::classes::a_star_grid_2d::CellShape;
use godot::classes::a_star_grid_2d::DiagonalMode;
use godot::classes::a_star_grid_2d::Heuristic;

use crate::player::Player;
use crate::wolf::Wolf;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct TileMapManager {
	tilemap: Option<Gd<TileMapLayer>>,
	nav: Option<Gd<AStarGrid2D>>,
	base: Base<Node2D>
}

#[godot_api]
impl INode2D for TileMapManager {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			tilemap: None,
			nav: None,
			base
		}
	}
	
	fn ready(&mut self) {
		self.initialise_tilemap();
		self.initialise_pathfinding();
		
		// Initialise all entities within the tilemap.
		let mut tree = self.base().get_tree().unwrap();
		let entities = tree.get_nodes_in_group("entities");
		
		for node in entities.iter_shared() {
			self.lock_to_grid(&node);
			self.register_signals(&node);
		}		
	}
}

impl TileMapManager {
	fn initialise_tilemap(&mut self) {
		let tilemap : Gd<TileMapLayer> = self.base().get_node_as("TerrainLayer");
		self.tilemap = Some(tilemap);
	}
	
	fn initialise_pathfinding(&mut self) {
		let tilemap = self.tilemap.as_ref().unwrap();
		
		// Basic configuration.
		let mut nav : Gd<AStarGrid2D> = AStarGrid2D::new_gd();
		nav.set_cell_shape(CellShape::ISOMETRIC_RIGHT);
		nav.set_diagonal_mode(DiagonalMode::NEVER);
		nav.set_default_compute_heuristic(Heuristic::MANHATTAN);

		// Set the correct tile size.
		let tileset = tilemap.get_tile_set().unwrap();
		let tile_size = tileset.get_tile_size().cast_float();
		nav.set_cell_size(tile_size);

		// Set the correct tilemap size.
		let limit = self.base().get_viewport_rect().size / tile_size;
		let region = Rect2i::new(-limit.cast_int(), limit.cast_int() * 2);
		nav.set_region(region);

		nav.update();
		self.nav = Some(nav);
		self.update_pathfinding();
	}
	
	fn update_pathfinding(&mut self) {
		let foreground : Gd<TileMapLayer> = self.base().get_node_as("ForegroundLayer");
		
		let mut tree = self.base().get_tree().unwrap();
		let tilemap =  self.tilemap.as_ref().unwrap();
		let nav = self.nav.as_mut().unwrap();
		
		// Mark all foreground tiles as impassable, since they represent walls and such.
		let foreground_tiles = foreground.get_used_cells();
		for tile in foreground_tiles.iter_shared() {
			nav.set_point_solid(tile);
		}
		
		// Mark all static "scenery" as impassable.
		let entities = tree.get_nodes_in_group("scenery");
		for entity in entities.iter_shared() {
			let node : Gd<Node2D> = entity.cast();
			let pos = node.get_position();
			let tile = global_to_grid(&tilemap, pos);
			nav.set_point_solid(tile);
		}	
	}
	
	/// Locks an entity's global position to the isometric grid of the tilemap.
	fn lock_to_grid(&self, node: &Gd<Node>) {
		let tilemap = self.tilemap.as_ref().unwrap();
	
		// First, convert the entity's global coordinates to grid coordinates.
		let mut node2d : Gd<Node2D> = node.clone().cast();
		let pos = node2d.get_position();
		let grid_pos = global_to_grid(&tilemap, pos);
		
		// Then convert them back into global coordinates.
		let new_pos = grid_to_global(&tilemap, grid_pos);
		node2d.set_position(new_pos);
	}
	
	/// Registers signal handlers.
	fn register_signals(&mut self, node: &Gd<Node>) {
		match node.get_class().to_string().as_str() {
			"Player" => self.register_player_signals(node.clone()),
			"Wolf" => self.register_wolf_signals(node.clone()),
			_ => ()
		};
	}
	
	fn register_player_signals(&mut self, node: Gd<Node>) {
		let player : Gd<Player> = node.cast();
		player.signals().reserve_tile().connect_other(self, Self::on_reserve_tile);
		player.signals().unreserve_tile().connect_other(self, Self::on_unreserve_tile);
		player.signals().update_nav().connect_other(self, Self::on_update_nav_player);
	}
	
	fn register_wolf_signals(&mut self, node: Gd<Node>) {
		let wolf : Gd<Wolf> = node.cast();
		wolf.signals().reserve_tile().connect_other(self, Self::on_reserve_tile);
		wolf.signals().unreserve_tile().connect_other(self, Self::on_unreserve_tile);
		wolf.signals().update_nav().connect_other(self, Self::on_update_nav_wolf);
	}
	
	fn on_reserve_tile(&mut self, tile: Vector2i) {
		let nav = self.nav.as_mut().unwrap();
		nav.set_point_solid(tile);
	}
	
	fn on_unreserve_tile(&mut self, tile: Vector2i) {
		let nav = self.nav.as_mut().unwrap();
		nav.set_point_solid_ex(tile).solid(false).done();
	}
	
	fn on_update_nav_player(&mut self, mut instance: Gd<Player>) {
		let tilemap = self.tilemap.as_ref().unwrap();
		instance.bind_mut().character.set_tilemap(tilemap.clone());
		let nav = self.nav.as_mut().unwrap();
		instance.bind_mut().character.set_nav(nav.clone());
	}
	
	fn on_update_nav_wolf(&mut self, mut instance: Gd<Wolf>) {
		let tilemap = self.tilemap.as_ref().unwrap();
		instance.bind_mut().character.set_tilemap(tilemap.clone());
		let nav = self.nav.as_mut().unwrap();
		instance.bind_mut().character.set_nav(nav.clone());
	}
}

pub fn grid_to_global(tilemap: &TileMapLayer, coords: Vector2i) -> Vector2 {
	let local_coords = tilemap.map_to_local(coords);
	tilemap.to_global(local_coords)
}

pub fn global_to_grid(tilemap: &TileMapLayer, coords: Vector2) -> Vector2i {
	let local_coords = tilemap.to_local(coords);
	tilemap.local_to_map(local_coords)
}
