# isogame

A proof-of-concept isometric 2D game intended to teach myself how to write games for Godot entirely in Rust!

TODO:

- Hyperwalk when walking into an obstacle, because we don't check if the animation is already playing...
- Store level state between level transitions
	- Get everything in entities group, then save them in a key:value map where key is node name and value is position
- Improve the update_pathfinding() method

Assets:

- https://scrabling.itch.io/pixel-isometric-tiles
- https://vledic.itch.io/vledics-pixel-rpg-tavern

![gif of isogame](isogame.gif)
