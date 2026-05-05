use crate::data::Database;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Layout {
    pub positions: HashMap<String, Position>,
    pub width: f32,
    pub height: f32,
}

pub const NODE_W: f32 = 220.0;
pub const NODE_H: f32 = 80.0;
pub const HORIZ_GAP: f32 = 28.0;
pub const VERT_GAP: f32 = 70.0;

pub fn compute(db: &Database) -> Layout {
    let mut positions = HashMap::new();
    let total_w = layout_subtree(&db.root, db, 0, 0.0, &mut positions);
    let max_y = positions
        .values()
        .map(|p| p.y)
        .fold(0.0_f32, f32::max);
    Layout {
        positions,
        width: total_w,
        height: max_y + NODE_H,
    }
}

// Recursive layout: each subtree returns its total horizontal width.
// Parent is centered above its children's combined block.
fn layout_subtree(
    id: &str,
    db: &Database,
    depth: u32,
    x_off: f32,
    positions: &mut HashMap<String, Position>,
) -> f32 {
    let Some(person) = db.get(id) else {
        return 0.0;
    };
    let y = depth as f32 * (NODE_H + VERT_GAP);

    if person.children.is_empty() {
        positions.insert(id.to_string(), Position { x: x_off, y });
        return NODE_W;
    }

    let mut cur_x = x_off;
    let mut child_centers: Vec<f32> = Vec::new();
    for child_id in &person.children {
        let child_w = layout_subtree(child_id, db, depth + 1, cur_x, positions);
        child_centers.push(cur_x + child_w / 2.0);
        cur_x += child_w + HORIZ_GAP;
    }
    let children_block_w = cur_x - x_off - HORIZ_GAP;

    // Center this node above its children block.
    let first = *child_centers.first().unwrap();
    let last = *child_centers.last().unwrap();
    let center = (first + last) / 2.0;
    let parent_x = center - NODE_W / 2.0;
    positions.insert(id.to_string(), Position { x: parent_x, y });

    children_block_w.max(NODE_W)
}
