use godot::prelude::*;
use godot::classes::Node2D;
use godot::classes::INode2D;
use godot::classes::TileMapLayer;
use godot::classes::AStarGrid2D;
use godot::classes::a_star_grid_2d::CellShape;
use godot::classes::a_star_grid_2d::DiagonalMode;
use godot::classes::a_star_grid_2d::Heuristic;

use crate::character::Character;
use crate::player::Player;

/// Responsible for managing the isometric tilemap and the entities within it.
#[derive(GodotClass)]
#[class(base=Node2D)]
struct TileMapManager {
	tilemap: Option<Gd<TileMapLayer>>,
	nav: Option<Gd<AStarGrid2D>>,
	reserved_tiles: Vec<Vector2i>,
	base: Base<Node2D>
}

#[godot_api]
impl INode2D for TileMapManager {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			tilemap: None,
			nav: None,
			reserved_tiles: Vec::new(),
			base
		}
	}
	
	fn ready(&mut self) {
		// Store a pointer to the tilemap.
		let tilemap : Gd<TileMapLayer> = self.base().get_node_as("TerrainLayer");
		self.tilemap = Some(tilemap);
		
		self.nav = Some(self.initialise_pathfinding());
		
		let nav = self.nav.as_mut().unwrap();
		let tilemap = self.tilemap.as_ref().unwrap();
		
		let start = tilemap_to_astar(Vector2i::new(-14, 14));
		let end = tilemap_to_astar(Vector2i::new(-6, 27));
		godot_print!("Path: {}, {}", &start, &end);
		let path = nav.get_id_path(start, end);
		godot_print!("{:?}", path);
		
		// Initialise all entities within the tilemap.
		let mut tree = self.base().get_tree().unwrap();
		let entities = tree.get_nodes_in_group("entities"); // TODO this is only done once at startup - what if more entities appear???
		
		for node in entities.iter_shared() {
			self.lock_entity(&node);
			self.register_tile_signal(&node);
		}
	}
}

impl TileMapManager {
	fn initialise_pathfinding(&self) -> Gd<AStarGrid2D> {
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
		let origin = Vector2i::ZERO;
		let limit = self.base().get_viewport_rect().size / tile_size;
		let region = Rect2i::new(origin, limit.cast_int());
		nav.set_region(region);
		
		nav.update();
		
		// Mark solid tiles as impassable
		let foreground : Gd<TileMapLayer> = self.base().get_node_as("ForegroundLayer");
		let foreground_tiles = foreground.get_used_cells();
		
		for tile in foreground_tiles.iter_shared() {
			let astar_tile = tilemap_to_astar(tile);
			nav.set_point_solid(astar_tile);
		}
		
		nav
	}

	/// Locks an entity's global position to the isometric grid of the tilemap.
	fn lock_entity(&self, node: &Gd<Node>) {
		let tilemap = self.tilemap.as_ref().unwrap();
	
		// First, convert the entity's global coordinates to grid coordinates.
		let mut node2d : Gd<Node2D> = node.clone().cast();
		let pos = node2d.get_position();
		let local_pos = tilemap.to_local(pos);
		let grid_pos = tilemap.local_to_map(local_pos);
		
		// Then convert them back into global coordinates.
		let mut new_pos = tilemap.map_to_local(grid_pos);
		new_pos = tilemap.to_global(new_pos);
		node2d.set_position(new_pos);
	}
	
	/// Registers signal handlers for the `on_reserve` and `on_unreserve` signals.
	fn register_tile_signal(&mut self, node: &Gd<Node>) {
		match node.get_class().to_string().as_str() {
			"Player" => self.register_player_signals(node.clone()),
			_ => ()
		};
	}
	
	fn register_player_signals(&mut self, node: Gd<Node>) {
		let player : Gd<Player> = node.cast();
		let reserve = player.signals().reserve_tile();
		let unreserve = player.signals().unreserve_tile();
		reserve.connect_other(self, Self::on_reserve_player);
		unreserve.connect_other(self, Self::on_unreserve_player);
	}
	
	/// When the `reserve_tile` signal is received, check whether the player is allowed to move.
	/// If they are, mark their destination tile as reserved and invoke `start_moving()`.
	fn on_reserve_player(&mut self, mut instance: Gd<Player>) {
		let tilemap = self.tilemap.as_ref().unwrap();
		let nav = self.nav.as_mut().unwrap();
		
		let mut player = instance.bind_mut();
		let coords = player.calculate_destination();
	
		// Check if the tile is currently occupied.
		let local_pos = tilemap.to_local(coords);
		let grid_pos = tilemap.local_to_map(local_pos);
		
		if self.reserved_tiles.contains(&grid_pos) {
			return;
		}
		
		self.reserved_tiles.push(grid_pos);
		nav.set_point_solid(tilemap_to_astar(grid_pos));
		player.start_moving();
	}
	
	/// When the `unreserve_tile` signal is received, remove all matching tiles from the internal `reserved_tiles` vector.
	fn on_unreserve_player(&mut self, coords: Vector2) {
		let tilemap = self.tilemap.as_ref().unwrap();
		let nav = self.nav.as_mut().unwrap();
		
		let local_pos = tilemap.to_local(coords);
		let grid_pos = tilemap.local_to_map(local_pos);
		
		self.reserved_tiles.retain(|i| {
			if *i == grid_pos {
				nav.set_point_solid_ex(tilemap_to_astar(grid_pos)).solid(false).done();
				return false; 
			}
			true
		});
	}
}

fn tilemap_to_astar(tilemap_coords: Vector2i) -> Vector2i {
	tilemap_coords * Vector2i::new(-1, 1)
}
