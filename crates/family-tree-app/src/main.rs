mod data;
mod layout;

use data::Database;
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
    let parsed: Database = serde_json::from_str(TREE_JSON).expect("tree.json must parse");
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
                <span class="subtitle">"The Curie family — a sample dataset"</span>
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
    let pad = 32.0_f32;
    let view_w = layout.width + pad * 2.0;
    let view_h = layout.height + pad * 2.0;

    let edges = build_edges(&layout, pad);
    let cards = build_cards(&layout, pad, selected);

    view! {
        <div class="tree-svg">
            <svg
                width=format!("{}", view_w)
                height=format!("{}", view_h)
                viewBox=format!("0 0 {} {}", view_w, view_h)
            >
                <g class="edges">
                    {edges}
                </g>
                <g class="nodes">
                    {cards}
                </g>
            </svg>
        </div>
    }
}

fn build_edges(layout: &layout::Layout, pad: f32) -> Vec<AnyView> {
    let mut out: Vec<AnyView> = Vec::new();
    for (id, person) in &db().people {
        let Some(parent_pos) = layout.positions.get(id) else {
            continue;
        };
        let px = parent_pos.x + NODE_W / 2.0 + pad;
        let py = parent_pos.y + NODE_H + pad;
        for child_id in &person.children {
            let Some(child_pos) = layout.positions.get(child_id) else {
                continue;
            };
            let cx = child_pos.x + NODE_W / 2.0 + pad;
            let cy = child_pos.y + pad;
            let mid_y = (py + cy) / 2.0;
            let path = format!("M {} {} V {} H {} V {}", px, py, mid_y, cx, cy);
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
    }
    out
}

fn build_cards(
    layout: &layout::Layout,
    pad: f32,
    selected: RwSignal<Option<String>>,
) -> Vec<AnyView> {
    let mut out: Vec<AnyView> = Vec::new();
    for (id, person) in &db().people {
        let Some(pos) = layout.positions.get(id) else {
            continue;
        };
        let id_owned = id.clone();
        let id_for_click = id.clone();
        let id_for_class = id.clone();
        let translate = format!("translate({}, {})", pos.x + pad, pos.y + pad);
        let dates = db().format_dates(&id_owned);
        let spouse_label = person
            .spouse
            .as_ref()
            .and_then(|sp_id| db().get(sp_id))
            .map(|sp| format!("m. {}", sp.name))
            .unwrap_or_default();
        let name = person.name.clone();

        let is_selected = move || selected.get().as_deref() == Some(id_for_class.as_str());

        out.push(
            view! {
                <g
                    transform=translate
                    class="person-card"
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
                        fill="#ffffff"
                        stroke="#475569"
                        stroke-width="1.5"
                    />
                    <text x="14" y="26" font-size="15" font-weight="600" fill="#0f172a">
                        {name}
                    </text>
                    <text x="14" y="48" font-size="13" fill="#475569">
                        {dates}
                    </text>
                    <text x="14" y="68" font-size="12" fill="#64748b">
                        {spouse_label}
                    </text>
                </g>
            }
            .into_any(),
        );
    }
    out
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
