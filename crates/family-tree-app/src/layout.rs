use crate::data::Database;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Layout {
    pub positions: HashMap<String, Position>,
    /// Couples drawn with a horizontal marriage line. Each entry is the pair of person ids
    /// (left, right) that should be linked.
    pub couples: Vec<(String, String)>,
    /// Family connector edges: ((parent_a, parent_b), child) — renderer draws line from
    /// the couple's midpoint down to the child's top-center. parent_b is None for single parents.
    pub family_edges: Vec<((String, Option<String>), String)>,
    pub width: f32,
    pub height: f32,
}

pub const NODE_W: f32 = 220.0;
pub const NODE_H: f32 = 88.0;
const COUPLE_GAP: f32 = 28.0;
const SUBTREE_GAP: f32 = 72.0;
const SIBLING_GAP: f32 = 36.0;
const VERT_GAP: f32 = 84.0;

/// Lays out an ancestor + descendant chart centered on `db.focus`.
///
/// Generation 0 (bottom): focus + siblings (children of focus's parents).
/// Generation -1: focus's parents (couple).
/// Generation -2: each parent's parents (two couples).
/// Generation +1: focus's children (descendants below focus).
///
/// The current implementation handles up to 2 generations of ancestors and 1 of descendants.
/// Extending depth requires generalizing `layout_ancestor_couple` to recurse.
pub fn compute(db: &Database) -> Layout {
    let mut positions = HashMap::new();
    let mut couples: Vec<(String, String)> = Vec::new();
    let mut family_edges: Vec<((String, Option<String>), String)> = Vec::new();

    let focus_id = db.focus.clone();
    let focus = match db.get(&focus_id) {
        Some(f) => f,
        None => {
            return Layout {
                positions,
                couples,
                family_edges,
                width: 0.0,
                height: 0.0,
            };
        }
    };

    // Step 1: identify focus's parents (the central couple).
    let central_parents: Vec<String> = focus.parents.clone();

    // Step 2: identify the bottom row — focus + siblings (children of any of the central parents).
    let bottom_row: Vec<String> = if !central_parents.is_empty() {
        let kids = db.children_of_any(&central_parents);
        if kids.is_empty() {
            vec![focus_id.clone()]
        } else {
            kids
        }
    } else {
        vec![focus_id.clone()]
    };

    // Step 3: for each central parent, identify their parents (grandparent couples).
    let mut grandparent_couples: Vec<(String, Option<(String, String)>)> = Vec::new();
    for parent_id in &central_parents {
        if let Some(parent) = db.get(parent_id) {
            let gp_pair = if parent.parents.len() == 2 {
                Some((parent.parents[0].clone(), parent.parents[1].clone()))
            } else {
                None
            };
            grandparent_couples.push((parent_id.clone(), gp_pair));
        }
    }

    // Step 4: compute widths.
    let bottom_width = if bottom_row.is_empty() {
        NODE_W
    } else {
        bottom_row.len() as f32 * NODE_W
            + (bottom_row.len().saturating_sub(1)) as f32 * SIBLING_GAP
    };

    let couple_block_w = 2.0 * NODE_W + COUPLE_GAP;

    // Top row width: sum of grandparent couple widths + SUBTREE_GAP between them.
    // If a parent has no grandparents, its slot collapses to NODE_W.
    let mut top_slots: Vec<f32> = Vec::new();
    for (_pid, gp) in &grandparent_couples {
        top_slots.push(if gp.is_some() { couple_block_w } else { NODE_W });
    }
    let top_width = if top_slots.is_empty() {
        0.0
    } else {
        top_slots.iter().sum::<f32>() + (top_slots.len().saturating_sub(1)) as f32 * SUBTREE_GAP
    };

    let total_width = top_width.max(bottom_width).max(couple_block_w);

    // Step 5: y coordinates per generation. Bottom row at the highest y.
    let n_gens: usize = 1
        + (if central_parents.is_empty() { 0 } else { 1 })
        + (if grandparent_couples
            .iter()
            .any(|(_, gp)| gp.is_some())
        {
            1
        } else {
            0
        });
    let y_top = 0.0;
    let row_step = NODE_H + VERT_GAP;
    let y_grandparents = y_top;
    let y_parents = if grandparent_couples.iter().any(|(_, gp)| gp.is_some()) {
        y_top + row_step
    } else {
        y_top
    };
    let y_bottom = y_top + (n_gens.saturating_sub(1)) as f32 * row_step;

    // Step 6: place the top row. Slots laid out left-to-right.
    let top_left = (total_width - top_width) / 2.0;
    let mut cursor_x = top_left;
    let mut parent_anchors: HashMap<String, f32> = HashMap::new(); // parent_id -> center_x
    for (i, (parent_id, gp_pair)) in grandparent_couples.iter().enumerate() {
        let slot_w = top_slots[i];
        match gp_pair {
            Some((gpa, gpb)) => {
                let left_x = cursor_x;
                let right_x = cursor_x + NODE_W + COUPLE_GAP;
                positions.insert(
                    gpa.clone(),
                    Position {
                        x: left_x,
                        y: y_grandparents,
                    },
                );
                positions.insert(
                    gpb.clone(),
                    Position {
                        x: right_x,
                        y: y_grandparents,
                    },
                );
                couples.push((gpa.clone(), gpb.clone()));
                family_edges.push((
                    (gpa.clone(), Some(gpb.clone())),
                    parent_id.clone(),
                ));
                let couple_mid_x = cursor_x + couple_block_w / 2.0;
                parent_anchors.insert(parent_id.clone(), couple_mid_x);
            }
            None => {
                // No grandparents known — anchor the parent at the slot center.
                let slot_mid_x = cursor_x + slot_w / 2.0;
                parent_anchors.insert(parent_id.clone(), slot_mid_x);
            }
        }
        cursor_x += slot_w;
        if i + 1 < top_slots.len() {
            cursor_x += SUBTREE_GAP;
        }
    }

    // Step 7: place the parent row. Each parent goes at their anchor x.
    if !central_parents.is_empty() {
        for parent_id in &central_parents {
            let anchor_x = *parent_anchors
                .get(parent_id)
                .unwrap_or(&(total_width / 2.0));
            let x = anchor_x - NODE_W / 2.0;
            positions.insert(
                parent_id.clone(),
                Position { x, y: y_parents },
            );
        }
        if central_parents.len() == 2 {
            couples.push((central_parents[0].clone(), central_parents[1].clone()));
        }
    }

    // Step 8: place the bottom row centered on the parents' midpoint (or on total_width/2 if no parents).
    let parent_mid_x = if central_parents.len() == 2 {
        let a = *parent_anchors.get(&central_parents[0]).unwrap_or(&0.0);
        let b = *parent_anchors.get(&central_parents[1]).unwrap_or(&0.0);
        (a + b) / 2.0
    } else if central_parents.len() == 1 {
        *parent_anchors
            .get(&central_parents[0])
            .unwrap_or(&(total_width / 2.0))
    } else {
        total_width / 2.0
    };

    let bottom_left = parent_mid_x - bottom_width / 2.0;
    for (i, sib_id) in bottom_row.iter().enumerate() {
        let x = bottom_left + i as f32 * (NODE_W + SIBLING_GAP);
        positions.insert(sib_id.clone(), Position { x, y: y_bottom });
        // Family edge: from central couple midpoint down to this child.
        if !central_parents.is_empty() {
            let p_a = central_parents[0].clone();
            let p_b = if central_parents.len() == 2 {
                Some(central_parents[1].clone())
            } else {
                None
            };
            family_edges.push(((p_a, p_b), sib_id.clone()));
        }
    }

    let height = y_bottom + NODE_H;

    Layout {
        positions,
        couples,
        family_edges,
        width: total_width,
        height,
    }
}
