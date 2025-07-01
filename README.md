# isogame

A proof-of-concept isometric 2D game intended to teach myself how to write games for Godot entirely in Rust!

TODO:

- Store level state between level transitions
- Improve the update_pathfinding() method
- Implement a slightly movement delay so it's possible to turn without moving
	- Do this by limiting how many times per second we check for input or try to move the wolf
- Implement death for the player

Assets:

- https://scrabling.itch.io/pixel-isometric-tiles
- https://vledic.itch.io/vledics-pixel-rpg-tavern

![gif of isogame](isogame.gif)
