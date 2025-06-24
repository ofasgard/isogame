use godot::prelude::*;

mod tilemap_manager;
mod player;
mod util;
mod wolf;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
