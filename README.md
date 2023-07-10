# Old School RuneScape Strategy Game 
A real-time strategy game that merges the immersive world of Old School RuneScape with the gameplay mechanics of strategy games like Age of Empires 2.

## Preview
The game can be compiled to WebAssembly and hosted on Github Pages.

See an example here: [https://markzuckerbergas.github.io/osrssg/](https://markzuckerbergas.github.io/osrssg/)

To explore the compiled WebAssembly game and the necessary HTML/JavaScript files that tie everything together, switch to the [web](https://github.com/markzuckerbergas/osrssg/tree/web) branch.

## Installation

Install Rust on your local machine
[The Rust Programming Language Book - Getting Started](https://doc.rust-lang.org/book/ch01-01-installation.html).

Run the game and optionally enable fast compiles.
``` bash
cargo run --features bevy/dynamic_linking 
```
[Reference](https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional)

## Currently working on

### Camera (Isometric viewpoint)
The camera needs to be constrained to an isometric projection.

### Defining camera movement
1) Player can move the camera around the map by moving the mouse to the edge of the screen or by using the arrow keys.
2) Player can zoom in and out by using the mouse wheel.
3) Player can move the camera to a specific location by clicking on the minimap.

### Left click selection
1) Player can select a single unit by left clicking on it.
2) Player can select multiple units by left clicking and dragging a selection box around them.
3) If units can move, they will move to the location that the player right clicked on.




