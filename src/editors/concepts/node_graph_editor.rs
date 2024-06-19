use std::cmp::max;
use std::collections::HashMap;
use std::iter::*;
use std::vec;
use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Painter, Sense, Ui};
use lazy_static::lazy_static;

const BODY_COLOR: egui::Color32 = egui::Color32::from_rgb(128, 128, 128);
const BODY_STROKE_COLOR: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
const LABEL_STROKE_COLOR: egui::Color32 = egui::Color32::from_rgb(8, 8, 8);

lazy_static! {
    //WHY is this initializer not const??
    static ref BODY_STROKE: egui::Stroke = egui::Stroke::new(1.0, BODY_STROKE_COLOR);
    static ref LABEL_STROKE: egui::Stroke = egui::Stroke::new(1.0, LABEL_STROKE_COLOR);
}

#[derive(Clone, Debug)]
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
            outputs: outputs.iter().map(|s| s.to_string()).zip(repeat(None)).collect()
        }
    }
    //Labels get created on the bottom for nodes that have both inputs and outputs.
    //For nodes with only inputs and no outputs, we put a vertical stripe on the right,
    //and for nodes with no inputs, we put a vertical stripe on the left.
    //TODO, add an icon to symbolize those (note for audio, whatever)
    fn draw(&self, painter: &Painter, pos: Pos2, cat_color: egui::Color32){

        let n_in = self.inputs.len();
        let n_out = self.outputs.len();

        //Input & Output
        if n_in > 0 && n_out > 0 {
            let body_rect = Rect::from_min_size(pos,
                                                vec2(self.name.len() as f32 * 10.0,
                                                     (max(n_in, n_out) as f32) *20.0));
            painter.rect(body_rect, egui::Rounding::ZERO, BODY_COLOR, *BODY_STROKE);

            let label_rect= Rect::from_two_pos(Pos2::from((body_rect.left_bottom().x,
                                                           body_rect.left_bottom().y+ 20.0)),
                                               body_rect.right_bottom());
            painter.rect(label_rect,
                         egui::Rounding::ZERO,
                         cat_color,
                         *LABEL_STROKE);
            painter.text(label_rect.center(),
                         egui::Align2::CENTER_CENTER,
                         self.name.clone(),
                         egui::FontId::default(),
                         egui::Color32::WHITE);
            return
        }
        //Body type for input only/output only
        let body_rect = Rect::from_min_size(pos,vec2(60.0, max(n_in, n_out) as f32) *20.0);
        painter.rect(body_rect, egui::Rounding::ZERO, BODY_COLOR, *BODY_STROKE);
        let mut label_rect= Rect::NOTHING;

        //Output only left-label
        if n_in == 0 {
            label_rect= Rect::from_two_pos(Pos2::from((body_rect.left_top().x-20.0,
                                                           body_rect.left_top().y)),
                                               body_rect.left_bottom());
        }
        //Input only right-label
        if n_out == 0 {
            label_rect= Rect::from_two_pos(Pos2::from((body_rect.right_top().x+20.0,
                                                           body_rect.right_top().y)),
                                               body_rect.right_bottom());
        }

        painter.rect(label_rect, egui::Rounding::ZERO, cat_color, *LABEL_STROKE);
        if(n_in == 0 && n_out == 0){
            panic!("UN-CONNECTABLE NODE CREATED SOMEHOW?")
        }
    }
}

pub struct NodeGraphEditor {
    id_prefix: &'static str,
    nodes: Vec<(Node, Pos2)>,
    next_node_id: usize,
    popup_open: bool,
    node_types: Vec<Node>,
    cat_map: HashMap<String, egui::Color32>,
    click_pos: Option<Pos2>
}

impl NodeGraphEditor {
    pub fn new(id_prefix: &'static str, node_types: Vec<Node>, cat_map: HashMap<String, egui::Color32>) -> Self {
        Self {
            id_prefix: id_prefix,
            nodes: vec![],
            next_node_id: 0,
            popup_open: false,
            node_types: node_types,
            cat_map: cat_map,
            click_pos: None,
        }
    }

    pub fn ui_content(&mut self, ctx: &Context, ui: &mut Ui) -> egui::Response {
        egui::SidePanel::left("ng_left_panel").show(ctx, |ui| {
            ui.label("Editors");
            if ui.button("Response Curves").clicked() {

            }
        });
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