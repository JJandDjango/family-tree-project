mod data;
mod layout;

use data::{Database, Person};
use layout::{NODE_H, NODE_W};
use leptos::prelude::*;
use std::sync::OnceLock;

const TREE_JSON: &str = include_str!("../public/tree.json");

static DB: OnceLock<Database> = OnceLock::new();

fn db() -> &'static Database {
    DB.get().expect("Database not initialized")
}

fn main() {
    console_error_panic_hook::set_once();
    let parsed = Database::parse(TREE_JSON).expect("tree.json must parse");
    DB.set(parsed).expect("DB already set");
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let selected: RwSignal<Option<String>> = RwSignal::new(None);
    view! {
        <div class="app">
            <header class="app-header">
                <h1>"Family Tree"</h1>
                <span class="subtitle">"Hyden — three generations"</span>
            </header>
            <div class="tree-container">
                <TreeSvg selected=selected />
                <SidePanel selected=selected />
            </div>
        </div>
    }
}

#[component]
fn TreeSvg(selected: RwSignal<Option<String>>) -> impl IntoView {
    let layout = layout::compute(db());
    let pad = 40.0_f32;
    let view_w = layout.width + pad * 2.0;
    let view_h = layout.height + pad * 2.0;

    let family_edges = build_family_edges(&layout, pad);
    let marriage_lines = build_marriage_lines(&layout, pad);
    let cards = build_cards(&layout, pad, selected);

    view! {
        <div class="tree-svg">
            <svg
                width=format!("{}", view_w)
                height=format!("{}", view_h)
                viewBox=format!("0 0 {} {}", view_w, view_h)
            >
                <g class="family-edges">
                    {family_edges}
                </g>
                <g class="marriage-lines">
                    {marriage_lines}
                </g>
                <g class="nodes">
                    {cards}
                </g>
            </svg>
        </div>
    }
}

fn build_family_edges(layout: &layout::Layout, pad: f32) -> Vec<AnyView> {
    let mut out: Vec<AnyView> = Vec::new();
    for ((parent_a, parent_b), child) in &layout.family_edges {
        let Some(a_pos) = layout.positions.get(parent_a) else {
            continue;
        };
        let Some(c_pos) = layout.positions.get(child) else {
            continue;
        };

        let parent_mid_x = match parent_b.as_ref().and_then(|b| layout.positions.get(b)) {
            Some(b_pos) => {
                let a_center = a_pos.x + NODE_W / 2.0;
                let b_center = b_pos.x + NODE_W / 2.0;
                (a_center + b_center) / 2.0
            }
            None => a_pos.x + NODE_W / 2.0,
        };

        let parent_y_bottom = a_pos.y + NODE_H + pad;
        let child_x = c_pos.x + NODE_W / 2.0 + pad;
        let child_y_top = c_pos.y + pad;
        let mid_y = (parent_y_bottom + child_y_top) / 2.0;
        let path = format!(
            "M {} {} V {} H {} V {}",
            parent_mid_x + pad,
            parent_y_bottom,
            mid_y,
            child_x,
            child_y_top
        );
        out.push(
            view! {
                <path
                    d=path
                    fill="none"
                    stroke="#94a3b8"
                    stroke-width="1.5"
                />
            }
            .into_any(),
        );
    }
    out
}

fn build_marriage_lines(layout: &layout::Layout, pad: f32) -> Vec<AnyView> {
    let mut out: Vec<AnyView> = Vec::new();
    for (a_id, b_id) in &layout.couples {
        let Some(a) = layout.positions.get(a_id) else {
            continue;
        };
        let Some(b) = layout.positions.get(b_id) else {
            continue;
        };
        let mid_y = a.y + NODE_H / 2.0 + pad;
        let line_left = a.x + NODE_W + pad;
        let line_right = b.x + pad;
        out.push(
            view! {
                <line
                    x1=line_left
                    y1=mid_y
                    x2=line_right
                    y2=mid_y
                    stroke="#cbd5e1"
                    stroke-width="2"
                    stroke-dasharray="4 3"
                />
            }
            .into_any(),
        );
    }
    out
}

fn build_cards(
    layout: &layout::Layout,
    pad: f32,
    selected: RwSignal<Option<String>>,
) -> Vec<AnyView> {
    let mut out: Vec<AnyView> = Vec::new();
    // Iterate in a stable order so SVG draw order is deterministic.
    let mut ids: Vec<&String> = layout.positions.keys().collect();
    ids.sort();
    for id in ids {
        let Some(pos) = layout.positions.get(id) else {
            continue;
        };
        let Some(person) = db().get(id) else {
            continue;
        };
        let id_owned = id.clone();
        let id_for_click = id.clone();
        let id_for_class = id.clone();
        let translate = format!("translate({}, {})", pos.x + pad, pos.y + pad);
        let dates = db().format_dates(&id_owned);
        let role = card_role(person);
        let name = person.name.clone();
        let is_focus = id_owned == db().focus;

        let is_selected = move || selected.get().as_deref() == Some(id_for_class.as_str());

        let card_class = if is_focus {
            "person-card focus"
        } else if person.deceased {
            "person-card deceased"
        } else if person.anonymized {
            "person-card anonymized"
        } else {
            "person-card"
        };

        out.push(
            view! {
                <g
                    transform=translate
                    class=card_class
                    class:selected=is_selected
                    on:click=move |_| {
                        let id = id_for_click.clone();
                        selected.update(|cur| {
                            *cur = if cur.as_deref() == Some(id.as_str()) {
                                None
                            } else {
                                Some(id)
                            };
                        });
                    }
                >
                    <rect
                        class="card-bg"
                        width=NODE_W
                        height=NODE_H
                        rx="10"
                        ry="10"
                    />
                    <text x="14" y="26" font-size="15" font-weight="600" class="card-name">
                        {name}
                    </text>
                    <text x="14" y="48" font-size="13" class="card-dates">
                        {dates}
                    </text>
                    <text x="14" y="68" font-size="12" class="card-role">
                        {role}
                    </text>
                </g>
            }
            .into_any(),
        );
    }
    out
}

fn card_role(person: &Person) -> String {
    if let Some(place) = &person.place {
        return place.clone();
    }
    if person.anonymized {
        return "(living)".to_string();
    }
    if person.deceased {
        return String::new();
    }
    String::new()
}

#[component]
fn SidePanel(selected: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <aside class="side-panel" class:open=move || selected.get().is_some()>
            {move || {
                selected.get().and_then(|id| {
                    let person = db().get(&id)?.clone();
                    let dates = db().format_dates(&id);
                    let spouse_name = person
                        .spouse
                        .as_ref()
                        .and_then(|sp| db().get(sp))
                        .map(|sp| sp.name.clone());
                    let place = person.place.clone();
                    let notes = person.notes.clone();
                    Some(view! {
                        <div class="panel-content">
                            <button
                                class="close-btn"
                                on:click=move |_| selected.set(None)
                            >
                                "×"
                            </button>
                            <h2>{person.name.clone()}</h2>
                            <p class="dates">{dates}</p>
                            {place.map(|pl| view! {
                                <span class="meta-label">"Place"</span>
                                <p class="meta">{pl}</p>
                            })}
                            {spouse_name.map(|n| view! {
                                <span class="meta-label">"Spouse"</span>
                                <p class="meta">{n}</p>
                            })}
                            {notes.map(|n| view! {
                                <div class="notes">
                                    <span class="meta-label">"Notes"</span>
                                    <p>{n}</p>
                                </div>
                            })}
                        </div>
                    })
                })
            }}
        </aside>
    }
}
