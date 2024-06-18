use std::cmp::max;
use std::collections::HashMap;
use std::iter::*;
use std::vec;
use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Painter, Sense, Ui};

#[derive(Clone)]
pub struct Node {
    pub id: Option<usize>,
    pub name: String,
    pub category: String,
    pub inputs: Vec<(String,Option<Node>)>,
    pub outputs: Vec<(String,Option<Node>)>,
}

impl Node {
    pub fn new(id: Option<usize>,
               name: &str,
               category: &str,
               inputs: Vec<&str>,
               outputs: Vec<&str>) -> Self {
        Self {
            id,
            name: name.to_string(),
            category: category.to_string(),
            inputs: inputs.iter().map(|s| s.to_string()).zip(repeat(None)).collect(),
            outputs: inputs.iter().map(|s| s.to_string()).zip(repeat(None)).collect()
        }
    }
    fn draw(&self, painter: &Painter, pos: Pos2, cat_color: egui::Color32){
        let n_in = self.inputs.len();
        let n_out = self.outputs.len();
        if(n_in > 0 && n_out > 0){
            let body_rect = Rect::from_min_size(pos,
                                                vec2(self.name.len()as f32 * 10.0,
                                                     (max(self.inputs.len(),
                                                          self.outputs.len()) as f32) *20.0));

            painter.rect(body_rect, egui::Rounding::ZERO,
                         egui::Color32::from_rgb(128, 128, 128),
                         egui::Stroke::new(1.0,
                                           egui::Color32::from_rgb(160, 160, 160)));

            let label_rect= Rect::from_two_pos(Pos2::from((body_rect.left_bottom().x,
                                                           body_rect.left_bottom().y+ 20.0)),
                                               body_rect.right_bottom());
            painter.rect(label_rect,
                         egui::Rounding::ZERO,
                         cat_color,
                         egui::Stroke::new(1.0,
                                           egui::Color32::from_rgb(160, 160, 160)));
            painter.text(label_rect.center(),
                         egui::Align2::CENTER_CENTER,
                         self.name.clone(),
                         egui::FontId::default(),
                         egui::Color32::WHITE);
        }

    }
}

pub struct NodeGraphEditor {
    nodes: Vec<(Node, Pos2)>,
    next_node_id: usize,
    popup_open: bool,
    node_types: Vec<Node>,
    cat_map: HashMap<String, egui::Color32>,
    click_pos: Option<Pos2>
}

impl NodeGraphEditor {
    pub fn new(node_types: Vec<Node>, cat_map: HashMap<String, egui::Color32>) -> Self {
        Self {
            nodes: vec![],
            next_node_id: 0,
            popup_open: false,
            node_types: node_types,
            cat_map: cat_map,
            click_pos: None,
        }
    }

    pub fn ui_content(&mut self, ctx: &Context, ui: &mut Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(500.0, 300.0),
                                Sense::drag().union(Sense::click()));

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, vec2(1.0,1.0)),
            response.rect,
        );

        if response.secondary_clicked() { //steals clicks from the context menu. whatever
            self.click_pos = ui.ctx().pointer_interact_pos();
        }

        response.context_menu(|ui| {
            let mut categories = self.node_types.iter()
                                                              .map(|n| n.category.clone())
                                                              .collect::<Vec<_>>();
            categories.dedup();

            for category in categories {
                ui.menu_button(category.clone(), |ui| {
                    for node_type in &self.node_types {
                        if node_type.category == category {
                            if ui.button(node_type.name.clone()).clicked() {
                                if let Some(pos) = self.click_pos {
                                    self.nodes.push((node_type.clone(), pos));
                                    self.next_node_id += 1;
                                    ui.close_menu();
                                }
                            }
                        }
                    }
                });
            }
        });

        // Draw nodes
        for node in &self.nodes {
            node.0.draw(&painter, node.1, self.cat_map[&node.0.category]);
        }

        response
    }
}