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

The game can be compiled to WebAssembly for web deployment. A comprehensive build system is set up for optimal web performance.

## 🌐 Web Deployment

### Prerequisites

Install the required tools for WASM compilation:

```bash
# Install the WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen for JavaScript bindings
cargo install wasm-bindgen-cli

# Install wasm-opt for size optimization (via binaryen)
brew install binaryen  # macOS
# or apt install binaryen  # Ubuntu
# or choco install binaryen  # Windows
```

### Building for Web

The project includes optimized build profiles for WASM deployment:

```bash
# Build optimized WASM binary
cargo build --profile wasm-release --target wasm32-unknown-unknown

# Generate JavaScript bindings
wasm-bindgen --out-dir wasm --target web --no-typescript target/wasm32-unknown-unknown/wasm-release/osrssg.wasm

# Optimize WASM file size (reduces from ~20MB to ~18MB)
wasm-opt -O --enable-bulk-memory-opt --enable-nontrapping-float-to-int --enable-reference-types --enable-sign-ext -o wasm/osrssg_bg_optimized.wasm wasm/osrssg_bg.wasm
```

### Optimization Features

The `Cargo.toml` includes specialized profiles for maximum size reduction:

- **`wasm-release` profile**: Optimized specifically for web deployment
- **Size optimization**: `opt-level = 'z'` for minimum binary size
- **Link Time Optimization**: `lto = true` for better optimization
- **Single codegen unit**: `codegen-units = 1` for improved LTO
- **Panic behavior**: `panic = 'abort'` to reduce binary size
- **Symbol stripping**: `strip = true` to remove debug symbols

### Deployment Results

The optimized build achieves significant size reductions:
- **Original build**: ~25MB WASM file
- **Optimized build**: ~18MB WASM file  
- **Size reduction**: 28% smaller, faster loading

### GitHub Pages Setup

The `web` branch contains the optimized deployment:

1. **Automated GitHub Pages**: Deploys from the `web` branch
2. **Optimized assets**: Includes compressed WASM and updated JavaScript
3. **Enhanced HTML**: Improved loading states and responsive design
4. **Asset management**: Proper handling of game assets and textures

### Local Testing

Test the web build locally:

```bash
# Navigate to your built web files
cd wasm/

# Start a local server
python3 -m http.server 8080

# Open http://localhost:8080 in your browser
```

### Deployment Workflow

1. **Make changes** on the `main` branch
2. **Build optimized WASM** using the commands above  
3. **Switch to `web` branch** and update with optimized files
4. **Commit and push** to trigger GitHub Pages deployment

## 🤝 Contributing

1. **Code Style**: Follow Rust conventions and add comments for complex logic
2. **Testing**: Test your changes with `cargo run`
3. **Documentation**: Update README if adding new features
4. **Modularity**: Keep systems focused on single responsibilities

## 🎯 Next Steps & TODOs

### Immediate Improvements
- [x] Multiple unit selection (AoE2-style tile-based drag selection box)
- [ ] Unit health and combat system
- [x] Resource gathering mechanics
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




