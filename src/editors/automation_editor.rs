use std::collections::HashMap;
use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Sense, Ui, Context, Id};
use crate::editors::concepts::node_graph_editor::*;
use crate::editors::concepts::nodes::*;

fn create_archetypes() -> Vec<NodeArchetype> {
    vec![
        NodeArchetype::new("Audio", cat_map("Sources"),
                  vec![],
                  vec!["RMS", "Pitch"]),
        NodeArchetype::new("Constant", cat_map("Sources"),
                  vec![],
                  vec!["0"]),

        NodeArchetype::new("x+y", cat_map("Arithmetic"),
                  vec!["x", "y"],
                  vec![""]),
        NodeArchetype::new("x-y", cat_map("Arithmetic"),
                  vec!["x", "y"],
                  vec![""]),
        NodeArchetype::new("x*y", cat_map("Arithmetic"),
                  vec!["x", "y"],
                  vec![""]),
        NodeArchetype::new("x/y", cat_map("Arithmetic"),
                  vec!["x", "y"],
                  vec![""]),
        NodeArchetype::new("x%y", cat_map("Arithmetic"),
                  vec!["x", "y"],
                  vec![""]),
        NodeArchetype::new("-x", cat_map("Arithmetic"),
                  vec![""],
                  vec![""]),
        NodeArchetype::new("1/x", cat_map("Arithmetic"),
                  vec![""],
                  vec![""]),

        NodeArchetype::new("sin(x)", cat_map("Trig"),
                  vec![""],
                  vec![""]),
        NodeArchetype::new("cos(x)", cat_map("Trig"),
                  vec![""],
                  vec![""]),
        NodeArchetype::new("tan(x)", cat_map("Trig"),
                  vec![""],
                  vec![""]),
        NodeArchetype::new("cot(x)", cat_map("Trig"),
                  vec![""],
                  vec![""]),
        NodeArchetype::new("sec(x)", cat_map("Trig"),
                  vec![""],
                  vec![""]),
        NodeArchetype::new("csc(x)", cat_map("Trig"),
                  vec![""],
                  vec![""]),

        NodeArchetype::new("poltocar", cat_map("Coordinates"),
                  vec!["r","θ"],
                  vec!["x","y"]),
        NodeArchetype::new("cartopol", cat_map("Coordinates"),
                  vec!["x","y"],
                  vec!["r","θ"]),
        NodeArchetype::new("sphertocar", cat_map("Coordinates"),
                  vec!["ρ","θ","φ"],
                  vec!["x","y","z"]),
        NodeArchetype::new("cartospher", cat_map("Coordinates"),
                  vec!["x","y","z"],
                  vec!["ρ","θ","φ"]),
    ]
}

fn cat_map(s: &'static str) -> (&'static str, egui::Color32, NodeType) {
    match s{
        "Sources" => (s, egui::Color32::from_rgb(200,190,215), NodeType::SOURCE),
        "Arithmetic" => (s, egui::Color32::from_rgb(225,170,170), NodeType::EFFECT),
        "Trig" => (s, egui::Color32::from_rgb(170,200,150), NodeType::EFFECT),
        "Coordinates" => (s, egui::Color32::from_rgb(170,190,225), NodeType::EFFECT),
        _ => panic!("category does not have entry in cat_map")
    }
}

pub struct AutomationEditor {
    node_graph_editor: NodeGraphEditor
}

impl Default for AutomationEditor {
    fn default() -> Self {
        Self {
            node_graph_editor: NodeGraphEditor::new(create_archetypes()),
        }
    }
}

impl AutomationEditor {
    pub fn ui_content(&mut self, ctx: &Context) {
        self.node_graph_editor.ui_content(ctx)
    }
}