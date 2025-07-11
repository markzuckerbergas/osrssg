use crate::components::*;
use bevy::prelude::*;

/// Sets up the game UI without spawn button
pub fn setup_game_ui(mut commands: Commands) {
    info!("🎨 Setting up game UI");

    // Setup inventory UI (hidden by default)
    setup_inventory_ui(&mut commands);
}

/// Sets up the OSRS-style inventory UI with 4x7 grid (28 slots)
fn setup_inventory_ui(commands: &mut Commands) {
    info!("📦 Setting up inventory UI");

    // Create inventory root container (bottom-right, hidden by default)
    let inventory_root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0), // Bottom-right corner
                bottom: Val::Px(10.0),
                width: Val::Px(4.0 * 34.0 + 16.0), // 4 slots * 34px (32px + 2px spacing) + 16px padding
                height: Val::Px(7.0 * 34.0 + 16.0), // 7 rows * 34px + 16px padding
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.12, 0.08)), // Dark brown OSRS-style background
            BorderColor(Color::srgb(0.6, 0.5, 0.3)), // Gold-ish border
            GlobalZIndex(500), // Above most UI but below minimap
            InventoryRoot,
            InventoryBorder,
            Name::new("InventoryRoot"),
        ))
        .id();

    // Create 28 inventory slots in a 4x7 grid
    for row in 0..7 {
        let row_entity = commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(32.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();

        commands.entity(inventory_root).add_child(row_entity);

        for col in 0..4 {
            let slot_index = row * 4 + col;
            
            let slot_entity = commands
                .spawn((
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.18, 0.15)), // Slightly lighter than background
                    BorderColor(Color::srgb(0.4, 0.35, 0.25)), // Darker border for slots
                    InventorySlot { slot_index },
                    Name::new(format!("InventorySlot_{}", slot_index)),
                ))
                .id();

            commands.entity(row_entity).add_child(slot_entity);
        }
    }
}

/// Updates the inventory UI based on selection and inventory changes
pub fn update_inventory_ui(
    mut selection_events: EventReader<SelectionChanged>,
    mut inventory_events: EventReader<InventoryChanged>,
    selected_units_query: Query<Entity, (With<Selected>, With<Controllable>)>,
    mut inventory_root_query: Query<&mut Node, (With<InventoryRoot>, Without<InventorySlot>)>,
    mut inventory_slots_query: Query<(Entity, &InventorySlot, &mut BackgroundColor), With<InventorySlot>>,
    units_query: Query<&Inventory, With<Controllable>>,
    mut commands: Commands,
) {
    let mut should_update = false;
    let mut current_selected_units: Vec<Entity> = Vec::new();

    // Check for selection changes
    for event in selection_events.read() {
        should_update = true;
        current_selected_units = event.selected_units.clone();
    }

    // Check for inventory changes - use current selection state from query
    for _event in inventory_events.read() {
        should_update = true;
        // Get current selection from the Selected component query
        current_selected_units = selected_units_query.iter().collect();
    }

    // If no events but we need to ensure consistency, check if we have a selection
    let current_selection: Vec<Entity> = selected_units_query.iter().collect();
    if !should_update && !current_selection.is_empty() {
        current_selected_units = current_selection;
        should_update = true; // Force update to ensure UI is visible
    }

    if !should_update {
        return;
    }

    // Update inventory UI visibility and contents
    let Ok(mut inventory_root_node) = inventory_root_query.single_mut() else {
        warn!("⚠️ Inventory root node not found");
        return;
    };

    // Show inventory if exactly one unit is selected and it has an inventory
    if current_selected_units.len() == 1 {
        let selected_unit = current_selected_units[0];
        
        if let Ok(inventory) = units_query.get(selected_unit) {
            // Show the inventory UI
            inventory_root_node.display = Display::Flex;
            
            // Update all inventory slots
            for (slot_entity, inventory_slot, mut slot_bg_color) in inventory_slots_query.iter_mut() {
                let slot_index = inventory_slot.slot_index;
                
                if let Some(item_stack) = inventory.slots[slot_index] {
                    // Slot has an item - change background color and add text
                    *slot_bg_color = BackgroundColor(Color::srgba(
                        item_stack.id.ui_color()[0],
                        item_stack.id.ui_color()[1], 
                        item_stack.id.ui_color()[2],
                        0.3 // Semi-transparent item color
                    ));
                    
                    // Add or update quantity text
                    // Note: In a more complete implementation, we'd track and update existing text entities
                    // For now, we'll just add text if quantity > 1
                    commands.entity(slot_entity).with_children(|parent| {
                        if item_stack.qty > 1 {
                            parent.spawn((
                                Text::new(item_stack.qty.to_string()),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(2.0),
                                    right: Val::Px(2.0),
                                    ..default()
                                },
                            ));
                        }
                    });
                } else {
                    // Empty slot - reset to default appearance
                    *slot_bg_color = BackgroundColor(Color::srgb(0.2, 0.18, 0.15));
                    // Note: In a more complete implementation, we'd remove any existing text entities
                }
            }
            
            // Only log inventory updates when inventory actually changes, not every frame
            // info!("📦 Inventory UI updated for unit with {}/28 slots used", inventory.used_slots());
        } else {
            // Selected unit doesn't have inventory - hide UI
            inventory_root_node.display = Display::None;
        }
    } else {
        // No units selected or multiple units selected - hide inventory UI
        inventory_root_node.display = Display::None;
        if current_selected_units.is_empty() {
            info!("📦 Inventory UI hidden (no selection)");
        } else {
            info!("📦 Inventory UI hidden (selection: {} units)", current_selected_units.len());
        }
    }
}
