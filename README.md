# isogame

A WIP experimental isometric 2D game for learning purposes.

- Complete refactor to pass tilemaps and astargrid2d instance into the character nodes
	- Three channels of communication only between character and TileMapManager: reserve a tile, unreserve a tile, populate the per-character pointers to TileMapLayer and AStarGrid2D
	- Character nodes should be able to perform all their logic in a self-contained manner, without deferring some of it to a parent "manager" node
	- If they need data to execute that logic, such as information about the tilemap or pathfinding data structures, they should request a copy or a pointer from their parent **only**
	- Don't need to keep track of reserved tiles - just pass the AStarGrid2D directly and use that to determine whether a tile is collidable

Assets:

- https://scrabling.itch.io/pixel-isometric-tiles
- https://vledic.itch.io/vledics-pixel-rpg-tavern
