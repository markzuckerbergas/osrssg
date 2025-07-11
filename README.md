# Old School RuneScape Strategy Game (OSRSSG)

A real-time strategy game that merges the immersive world of Old School RuneScape with the gameplay mechanics of strategy games like Age of Empires 2.

## 🎮 Game Features

- **Precise Unit Selection**: Left-click anywhere on a character (legs, torso, head) to select them
- **AoE2-Style Tile-Based Selection**: Drag to create a selection box that selects units based on their tile positions (inspired by Age of Empires 2)
  - Uses tile-based logic for precise, predictable selection
  - Consistent with the grid-based movement system
  - Perfect for OSRS-style tile interaction combined with RTS precision
- **Movement Commands**: Right-click to move selected units  
- **Enhanced Camera System**: Classic RTS-style view with multiple control options
  - **Arrow Keys**: Manual camera movement (left/right/up/down in world coordinates)
  - **Edge Scrolling**: Move camera by moving mouse to screen edges
  - **Camera Bounds**: Prevents camera from moving too far from action area
  - **Fixed Zoom**: Standard zoom level for optimal gameplay visibility
  - **Minimap**: Bottom-right overlay showing game area, player positions, and camera viewport (Press M to toggle)
- **Smooth Animations**: Walking and idle animations for units

## 🛠️ Technology Stack

- **Engine**: [Bevy Engine](https://bevyengine.org/) (Rust game engine)
- **Graphics**: 3D rendering with isometric camera projection
- **Assets**: GLTF models with embedded animations

## 🚀 Quick Start

### Prerequisites

1. Install Rust: [rustup.rs](https://rustup.rs/)
2. Clone this repository

### Running the Game

```bash
# For faster compilation during development
cargo run --features bevy/dynamic_linking

# Or just run normally
cargo run
```

### Controls

- **Arrow Keys**: Move camera around the map (left/right/up/down in world coordinates)
- **Mouse Edge Scrolling**: Move mouse to screen edges to scroll camera
- **Left Click**: Select a unit (click anywhere on the character model)
- **Left Click + Drag**: Create a selection box to select multiple units
- **Right Click**: Move selected unit(s) to clicked location
- **M Key**: Toggle minimap visibility

## 📁 Project Structure

The codebase is organized for easy understanding and maintenance:

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Module declarations and exports
├── components/          # Game components (ECS data)
│   └── mod.rs          # Selected, Moving, Controllable, UnitAnimations
├── resources/           # Global game state
│   └── mod.rs          # GameState, CameraSettings
└── systems/             # Game logic systems
    ├── mod.rs          # System module exports
    ├── animation.rs    # Animation management
    ├── camera.rs       # Camera controls
    ├── input.rs        # Mouse/keyboard input handling
    ├── movement.rs     # Unit movement logic
    └── setup.rs        # Scene setup and initialization
```

### Key Concepts

**Components** define what entities have:
- `Selected`: Marks units as selected by player
- `Moving`: Marks units that are currently moving
- `Controllable`: Marks units that can be controlled
- `UnitAnimations`: Stores animation references

**Resources** store global state:
- `GameState`: Current movement destination
- `CameraSettings`: Enhanced camera parameters including bounds, edge scrolling, and zoom limits
- `MinimapSettings`: Configuration for future minimap implementation

**Systems** contain the game logic:
- Input handling (selection, multi-selection, and movement commands)
- Enhanced camera controls (movement, zoom, edge scrolling, and bounds)
- Unit movement and animation updates

## 🎯 For New Developers

### Understanding the Code

1. **Start with `main.rs`**: See how systems are registered and the game loop works
2. **Look at `setup.rs`**: Understand how the game scene is initialized
3. **Check `input.rs`**: See how player input is processed
4. **Explore `movement.rs`**: Learn how units move around

### Making Changes

- **Add new unit types**: Create new components in `components/mod.rs`
- **New game mechanics**: Add systems in the `systems/` folder
- **UI elements**: Add new systems for UI handling
- **Graphics**: Modify lighting and rendering in `setup.rs`

### Key Learning Resources

- [Bevy Book](https://bevy-cheatbook.github.io/) - Comprehensive Bevy guide
- [ECS Pattern](https://bevy-cheatbook.github.io/programming/ecs.html) - Understanding Entity-Component-System
- [Rust Book](https://doc.rust-lang.org/book/) - Learning Rust programming

## 📝 Development Features

### Fast Compilation

For faster development, use dynamic linking:
```bash
cargo run --features bevy/dynamic_linking
```

### WebAssembly Build

The game can be compiled to WebAssembly for web deployment:
```bash
# Build for web (requires additional setup)
# See: https://bevy-cheatbook.github.io/platforms/wasm.html
```

## 🤝 Contributing

1. **Code Style**: Follow Rust conventions and add comments for complex logic
2. **Testing**: Test your changes with `cargo run`
3. **Documentation**: Update README if adding new features
4. **Modularity**: Keep systems focused on single responsibilities

## 🎯 Next Steps & TODOs

### Immediate Improvements
- [x] Multiple unit selection (AoE2-style tile-based drag selection box)
- [ ] Unit health and combat system
- [ ] Resource gathering mechanics
- [ ] Building construction

### Camera Enhancements
- [x] Edge scrolling (move camera when mouse near screen edge)
- [x] Minimap for navigation
- [x] Camera bounds (prevent moving too far from action)

### Gameplay Features
- [ ] Different unit types (workers, soldiers, etc.)
- [ ] Fog of war
- [ ] AI opponents
- [ ] Save/load game state

## 📖 Web Demo

See the live demo: [https://markzuckerbergas.github.io/osrssg/](https://markzuckerbergas.github.io/osrssg/)

The web build source is available in the `web` branch.

---

**Happy coding!** 🦀 This project is designed to be approachable for developers new to Rust or game development. Each module has clear responsibilities, and the code prioritizes readability over clever optimizations.




