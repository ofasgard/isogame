use godot::prelude::*;

mod tilemap_manager;
mod player;
mod util;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
