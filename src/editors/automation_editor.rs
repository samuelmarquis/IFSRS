use std::collections::HashMap;
use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Sense, Ui, Context, Id};
use crate::editors::concepts::node_graph_editor::*;

fn get_node_types() -> Vec<Node> {
    vec![
        Node::new(None, "Audio", "Sources",
                  vec![],
                  vec!["RMS"]),
        Node::new(None, "Constant", "Sources",
                  vec![],
                  vec![""]),

        Node::new(None, "x+y", "Arithmetic",
                  vec!["x", "y"],
                  vec![""]),
        Node::new(None, "x-y", "Arithmetic",
                  vec!["x", "y"],
                  vec![""]),
        Node::new(None, "x*y", "Arithmetic",
                  vec!["x", "y"],
                  vec![""]),
        Node::new(None, "x/y", "Arithmetic",
                  vec!["x", "y"],
                  vec![""]),
        Node::new(None, "x%y", "Arithmetic",
                  vec!["x", "y"],
                  vec![""]),
        Node::new(None, "-x", "Arithmetic",
                  vec![""],
                  vec![""]),
        Node::new(None, "1/x", "Arithmetic",
                  vec![""],
                  vec![""]),

        Node::new(None, "sin(x)", "Trig",
                  vec![""],
                  vec![""]),
        Node::new(None, "cos(x)", "Trig",
                  vec![""],
                  vec![""]),
        Node::new(None, "tan(x)", "Trig",
                  vec![""],
                  vec![""]),
        Node::new(None, "cot(x)", "Trig",
                  vec![""],
                  vec![""]),
        Node::new(None, "sec(x)", "Trig",
                  vec![""],
                  vec![""]),
        Node::new(None, "csc(x)", "Trig",
                  vec![""],
                  vec![""]),

        Node::new(None, "poltocar", "Coordinates",
                  vec!["r","θ"],
                  vec!["x","y"]),
        Node::new(None, "cartopol", "Coordinates",
                  vec!["x","y"],
                  vec!["r","θ"]),
        Node::new(None, "sphertocar", "Coordinates",
                  vec!["ρ","θ","ϕ"],
                  vec!["x","y","z"]),
        Node::new(None, "cartospher", "Coordinates",
                  vec!["x","y","z"],
                  vec!["ρ","θ","ϕ"]),
    ]
}

fn get_cat_map() -> HashMap<String, egui::Color32> {
    HashMap::from([
        ("Sources".to_string(), egui::Color32::from_rgb(200,190,215)),
        ("Arithmetic".to_string(), egui::Color32::from_rgb(225,170,170)),
        ("Trig".to_string(), egui::Color32::from_rgb(170,200,150)),
        ("Coordinates".to_string(), egui::Color32::from_rgb(170,190,225)),
    ])
}

pub struct AutomationEditor {
    node_graph_editor: NodeGraphEditor
}

impl Default for AutomationEditor {
    fn default() -> Self {
        Self {
            node_graph_editor: NodeGraphEditor::new(get_node_types(), get_cat_map()),
        }
    }
}

impl AutomationEditor {
    pub fn ui_content(&mut self, ctx: &Context, ui: &mut Ui) -> egui::Response {
        self.node_graph_editor.ui_content(ctx,ui)
    }
}