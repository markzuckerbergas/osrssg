use bevy::prelude::*;

// === Resource Gathering System ===
// 
// This module implements a RuneScape-style resource gathering system with:
// - Mining: Copper ore (level 1), Tin ore (level 1) 
// - Woodcutting: Normal logs (level 1)
// - 28-slot inventories with 1 item per slot (no stacking)
// - Experience points matching OSRS values
// - State-machine driven gathering AI (Walking → Harvesting → Full)
//
// The system follows OSRS mechanics:
// - Copper/Tin ore: 17.5 XP each (level 1 mining)
// - Normal logs: 25 XP each (level 1 woodcutting)
// - No stacking: each slot holds exactly 1 resource
// - Close gather radius (1.5 units) 
//
// Key components:
// - ResourceNode: Marks gatherable resources in the world
// - Inventory: Per-unit 28-slot bag with 1 item per slot
// - GatherTask: State machine for gathering behavior
// - Capacity: Inventory limits (28 slots max)

/// Event emitted when a unit should gather from a resource node
#[derive(Event)]
pub struct GatherEvent {
    pub unit: Entity,
    pub resource: Entity,
}

/// Event emitted when unit selection changes
#[derive(Event)]
pub struct SelectionChanged {
    pub selected_units: Vec<Entity>,
}

/// Event emitted when a unit's inventory changes
#[derive(Event)]
pub struct InventoryChanged {
    pub unit: Entity,
}

/// Marks an entity as selected by the player
#[derive(Component)]
pub struct Selected;

/// Marks an entity as currently moving to a destination
#[derive(Component)]
pub struct Moving;

/// Component for entities that can be controlled by the player
#[derive(Component)]
pub struct Controllable;

/// Individual destination for each unit
#[derive(Component)]
pub struct Destination {
    pub target: Vec3,
}

/// Marks the main game camera
#[derive(Component)]
pub struct MainCamera;

/// Marks the minimap camera
#[derive(Component)]
pub struct MinimapCamera;

/// Marks the minimap UI container
#[derive(Component)]
pub struct MinimapUI;

/// Marks a player dot on the minimap
#[derive(Component)]
pub struct MinimapPlayerDot;

/// Marks the camera viewport indicator on the minimap
#[derive(Component)]
pub struct MinimapCameraViewport;

/// Marks the inventory UI root container
#[derive(Component)]
pub struct InventoryRoot;

/// Marks an inventory slot UI element
#[derive(Component)]
pub struct InventorySlot {
    pub slot_index: usize,
}

/// Marks the inventory border/background UI element
#[derive(Component)]
pub struct InventoryBorder;

/// Tracks drag selection state
#[derive(Component)]
pub struct DragSelection {
    pub start_pos: Vec2,
    pub current_pos: Vec2,
}

/// Marks the drag selection box UI element
#[derive(Component)]
pub struct DragSelectionBox;

/// Types of resources that can be gathered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    // Mining resources
    Copper,
    Tin,
    // Woodcutting resources  
    Wood,
}

impl ResourceKind {
    /// Get the display name for the resource
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceKind::Copper => "Copper rock",
            ResourceKind::Tin => "Tin rock", 
            ResourceKind::Wood => "Tree",
        }
    }

    /// Get the skill required to gather this resource
    pub fn required_skill(&self) -> &'static str {
        match self {
            ResourceKind::Copper | ResourceKind::Tin => "Mining",
            ResourceKind::Wood => "Woodcutting",
        }
    }

    /// Get the level required to gather this resource (RuneScape-like)
    pub fn required_level(&self) -> u8 {
        match self {
            ResourceKind::Copper => 1,  // Level 1 mining in OSRS
            ResourceKind::Tin => 1,     // Level 1 mining in OSRS
            ResourceKind::Wood => 1,    // Level 1 woodcutting in OSRS
        }
    }

    /// Get the experience gained per resource (RuneScape-like)
    pub fn experience_per_item(&self) -> f32 {
        match self {
            ResourceKind::Copper => 17.5, // OSRS copper ore XP
            ResourceKind::Tin => 17.5,     // OSRS tin ore XP  
            ResourceKind::Wood => 25.0,    // OSRS normal logs XP
        }
    }

    /// Get the base gather rate (items per second)
    pub fn base_gather_rate(&self) -> f32 {
        match self {
            ResourceKind::Copper => 1.0,   // Same rate as logs for testing
            ResourceKind::Tin => 1.0,      // Same rate as logs for testing
            ResourceKind::Wood => 1.2,     // Faster for woodcutting
        }
    }
}

/// Unique identifier for items in inventories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemId {
    // Mining ores
    CopperOre,
    TinOre,
    // Woodcutting logs
    Logs,
}

impl ItemId {
    /// Get the display name for the item
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemId::CopperOre => "Copper ore",
            ItemId::TinOre => "Tin ore",
            ItemId::Logs => "Logs",
        }
    }

    /// Get the maximum stack size for this item (1 per slot)
    pub fn max_stack_size(&self) -> u16 {
        match self {
            // Each slot holds exactly 1 resource (no stacking)
            ItemId::CopperOre => 1,  // 1 copper ore per slot
            ItemId::TinOre => 1,     // 1 tin ore per slot  
            ItemId::Logs => 1,       // 1 log per slot
        }
    }

    /// Get the item color for UI display
    pub fn ui_color(&self) -> [f32; 4] {
        match self {
            ItemId::CopperOre => [0.9, 0.6, 0.3, 1.0], // Bright copper/orange (matches world color)
            ItemId::TinOre => [0.8, 0.8, 0.9, 1.0],    // Bright silver-blue (matches world color)
            ItemId::Logs => [0.2, 0.8, 0.2, 1.0],      // Bright green (matches world color)
        }
    }
}

impl From<ResourceKind> for ItemId {
    fn from(resource: ResourceKind) -> Self {
        match resource {
            ResourceKind::Copper => ItemId::CopperOre,
            ResourceKind::Tin => ItemId::TinOre,
            ResourceKind::Wood => ItemId::Logs,
        }
    }
}

/// Represents a stack of items in an inventory slot
#[derive(Debug, Clone, Copy)]
pub struct ItemStack {
    pub id: ItemId,
    pub qty: u16,
}

impl ItemStack {
    pub fn new(id: ItemId, qty: u16) -> Self {
        Self { id, qty }
    }
}

/// Marks an entity as a resource node that can be harvested
#[derive(Component)]
pub struct ResourceNode {
    pub kind: ResourceKind,
    pub remaining: u32,
    pub gather_rate: f32,      // Items per second
    pub gather_radius: f32,    // How close units need to be to gather
}

impl ResourceNode {
    pub fn new(kind: ResourceKind, remaining: u32, gather_rate: f32, gather_radius: f32) -> Self {
        Self {
            kind,
            remaining,
            gather_rate,
            gather_radius,
        }
    }
}

/// Per-unit inventory with OSRS-style 28 slots
#[derive(Component)]
pub struct Inventory {
    pub slots: [Option<ItemStack>; 28],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [None; 28],
        }
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Try to add items to the inventory, returns the number of items actually added
    pub fn add_items(&mut self, item_id: ItemId, qty: u16, max_stack: u16) -> u16 {
        let mut remaining = qty;

        // First, try to add to existing stacks
        for slot in &mut self.slots {
            if remaining == 0 {
                break;
            }

            if let Some(stack) = slot {
                if stack.id == item_id && stack.qty < max_stack {
                    let can_add = (max_stack - stack.qty).min(remaining);
                    stack.qty += can_add;
                    remaining -= can_add;
                }
            }
        }

        // Then, try to add to empty slots
        for slot in &mut self.slots {
            if remaining == 0 {
                break;
            }

            if slot.is_none() {
                let stack_size = remaining.min(max_stack);
                *slot = Some(ItemStack::new(item_id, stack_size));
                remaining -= stack_size;
            }
        }

        qty - remaining
    }

    /// Check if the inventory is full
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|slot| slot.is_some())
    }

    /// Get the number of used slots
    pub fn used_slots(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count()
    }

    /// Get the total quantity of a specific item
    pub fn count_item(&self, item_id: ItemId) -> u32 {
        self.slots
            .iter()
            .filter_map(|slot| *slot)
            .filter(|stack| stack.id == item_id)
            .map(|stack| stack.qty as u32)
            .sum()
    }
}

/// Inventory capacity settings
#[derive(Component)]
pub struct Capacity {
    pub max_slots: u8,
    pub max_stack: u16,
}

impl Default for Capacity {
    fn default() -> Self {
        Self {
            max_slots: 28,   // OSRS-style 28 slots
            max_stack: 1,    // 1 item per slot (no stacking)
        }
    }
}

/// State of gathering process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatherState {
    Walking,     // Moving to resource node
    Harvesting,  // Currently gathering
    Full,        // Inventory is full, need to stop
}

/// Component attached when a unit is ordered to gather resources
#[derive(Component)]
pub struct GatherTask {
    pub target: Entity,
    pub timer: Timer,
    pub state: GatherState,
}

impl GatherTask {
    pub fn new(target: Entity, gather_rate: f32) -> Self {
        let interval = 1.0 / gather_rate; // Convert rate to interval in seconds
        Self {
            target,
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
            state: GatherState::Walking,
        }
    }
}

// === Missing Components for Existing Systems ===

/// Component for unit collision detection
#[derive(Component, Default)]
pub struct UnitCollision {
    pub radius: f32,
    pub allow_friendly_overlap: bool,
}

/// Component for collision radius
#[derive(Component)]
pub struct CollisionRadius {
    pub radius: f32,
}

impl Default for CollisionRadius {
    fn default() -> Self {
        Self { radius: 0.3 }
    }
}

/// Marks static obstacles in the scene
#[derive(Component)]
pub struct StaticObstacle;

/// Timer for tracking stuck units
#[derive(Component)]
pub struct StuckTimer {
    pub timer: f32,
    pub last_position: Vec3,
    pub stuck_threshold: f32,
}

impl Default for StuckTimer {
    fn default() -> Self {
        Self {
            timer: 0.0,
            last_position: Vec3::ZERO,
            stuck_threshold: 2.0,
        }
    }
}

/// Marks the primary target for a group of units
#[derive(Component)]
pub struct PrimaryTarget;

/// Animation player component for units - links an AnimationPlayer entity to its parent unit
#[derive(Component)]
pub struct UnitAnimationPlayer {
    pub unit_entity: Entity,
}
