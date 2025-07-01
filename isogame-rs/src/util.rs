use godot::prelude::*;
use godot::builtin::Vector2;
use godot::builtin::Vector2i;
use godot::classes::Input;

/// Tracks the different keyboard inputs registered with the game.
pub enum KeyboardInput {
	MoveNW,
	MoveNE,
	MoveSW,
	MoveSE
}

impl KeyboardInput {
	/// Get keypress from the input singleton.
	pub fn get_key() -> Option<Self> {
		let input = Input::singleton();
		if input.is_action_pressed("move_nw") { return Some(KeyboardInput::MoveNW); }
		if input.is_action_pressed("move_ne") { return Some(KeyboardInput::MoveNE); }
		if input.is_action_pressed("move_sw") { return Some(KeyboardInput::MoveSW); }
		if input.is_action_pressed("move_se") { return Some(KeyboardInput::MoveSE); }
		None		
	}
	
	/// Get keypress and convert it into an `IsometricFacing` type, if applicable.
	pub fn get_movement() -> Option<IsometricFacing> {
		match KeyboardInput::get_key() {
			Some(input) => match input {
				KeyboardInput::MoveNW => Some(IsometricFacing::NW),
				KeyboardInput::MoveNE => Some(IsometricFacing::NE),
				KeyboardInput::MoveSW => Some(IsometricFacing::SW),
				KeyboardInput::MoveSE => Some(IsometricFacing::SE),
			},
			None => None		
		}
	}
}

/// Represents one of the four cardinal directions in an isometric grid.
#[derive(GodotConvert, Var, Export, Clone, Default, Debug, PartialEq)]
#[godot(via = GString)]
pub enum IsometricFacing {
	#[default]
	NW,
	NE,
	SW,
	SE
}


impl IsometricFacing {
	pub fn from_movement_vector(vector: Vector2, tile_width: f32) -> Option<Self> {
		match vector {
			val if val == IsometricFacing::NW.get_movement_vector(tile_width) => Some(IsometricFacing::NW),
			val if val == IsometricFacing::NE.get_movement_vector(tile_width) => Some(IsometricFacing::NE),
			val if val == IsometricFacing::SW.get_movement_vector(tile_width) => Some(IsometricFacing::SW),
			val if val == IsometricFacing::SE.get_movement_vector(tile_width) => Some(IsometricFacing::SE),
			_ => None
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			IsometricFacing::NW => "nw".to_string(),
			IsometricFacing::NE => "ne".to_string(),
			IsometricFacing::SW => "sw".to_string(),
			IsometricFacing::SE => "se".to_string()
		}		
	}

	pub fn get_animation(&self, animation: &str) -> String { format!("{}_{}", self.to_string(), animation) }

	/// Get a directional movement vector with a magnitude of 1 isometric tile.
	pub fn get_movement_vector(&self, tile_width: f32) -> Vector2 {
		let vector = match self {
			// Isometric tiles are twice as wide as they are tall.
			// So movement is halved in the Y axis.
			IsometricFacing::NW => Vector2::new(-1.0, -0.5),
			IsometricFacing::NE => Vector2::new(1.0, -0.5),
			IsometricFacing::SW => Vector2::new(-1.0, 0.5),
			IsometricFacing::SE => Vector2::new(1.0, 0.5)
		};
		// Divide tile width by 2 because it's a vector.
		// We only want to move half a tile in the X axis, and half a tile in the Y axis.
		vector * (tile_width / 2.0)		
	}
}

/// Represents the result of an attempt to find a path to a target node or tile.

pub enum PathfindingResult {
	NoPath,
	ReachedTarget(Vector2i),	// Tile containing the target
	FoundPath(Vector2i),		// Next tile on the path
}
