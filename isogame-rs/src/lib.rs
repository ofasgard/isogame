use godot::prelude::*;

mod character;
mod control;
mod level;
mod level_manager;
mod player;
mod util;
mod wolf;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
