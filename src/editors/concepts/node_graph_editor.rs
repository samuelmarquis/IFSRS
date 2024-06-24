use std::cmp::max;
use std::collections::HashMap;
use std::iter::*;
use std::rc::Rc;
use std::vec;
use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Painter, Sense, Ui};
use lazy_static::lazy_static;
use nalgebra::point;
use slotmap::HopSlotMap;
use crate::editors::concepts::nodes::*;

const EDGE_COLOR: egui::Color32 = egui::Color32::from_rgb(255,255,255);
lazy_static! {
    static ref EDGE_STROKE: egui::Stroke = egui::Stroke::new(2.0, EDGE_COLOR);
}

pub type Nodes = HopSlotMap<NodeId, Node>;
pub type Terminals = HopSlotMap<TermId, Terminal>;

pub struct NodeGraphEditor {
    archetypes: Vec<NodeArchetype>,
    nodes: Nodes,
    terminals: Terminals,

    selected_node: Option<usize>,
    click_pos: Option<Pos2>,

    drag_start: Option<Pos2>,
    drag_target: Option<Pos2>,
}

impl NodeGraphEditor {
    pub fn new(archetypes: Vec<NodeArchetype>) -> Self {
        Self {
            archetypes: archetypes,
            nodes: Nodes::default(),
            terminals: Terminals::default(),
            selected_node: None,
            click_pos: None,
            drag_start: None,
            drag_target: None,
        }
    }

    pub fn ui_content(&mut self, ctx: &Context) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
            if self.selected_node.is_some(){

            }
            else{
                ui.label("Select a node to see properties"); //center this. or dont
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(),
                                              ui.available_height()),
                                    Sense::drag().union(Sense::click()));

            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, vec2(1.0, 1.0)),
                response.rect,
            );

            let xwidth = response.rect.max[0] - response.rect.min[0];
            let ywidth = response.rect.max[1] - response.rect.min[1];
            let scale: Vec2 = vec2(1.0 / xwidth, 1.0 / ywidth);

            //we want to spawn nodes where the initial right-click took place, as opposed to
            //where the mouse ends up in selecting an option from the drop-down
            if response.secondary_clicked() { //steals clicks from the context menu
                self.click_pos = ui.ctx().pointer_interact_pos();
            }

            response.context_menu(|ui| {
                let mut categories = self.archetypes.iter()
                    .map(|n| n.category)
                    .collect::<Vec<_>>();
                categories.dedup();

                for category in categories { ui.menu_button(category,|ui| {
                    for archetype in self.archetypes.iter()
                        .filter(|x|x.category == category)
                    {
                        if ui.button(archetype.name).clicked() {
                            if let Some(pos) = self.click_pos {
                                let x = self.nodes.insert(Node::new(archetype));
                                self.nodes[x].set_pos(pos);
                                for s in &archetype.inputs{
                                    self.terminals.insert(Terminal::new(s, TermType::IN));
                                }
                                for s in &archetype.outputs{
                                    self.terminals.insert(Terminal::new(s, TermType::OUT));
                                }
                                ui.close_menu();
                            }
                        }
                    }
                });}
            });

            let mut edging: bool = false;
            let mut stopped_edging: bool = false;


            for (i, (_, n)) in self.nodes.iter_mut().enumerate(){
                n.draw(&painter);
                let tpos = n.get_terminal_pos();
                for (j, t) in tpos.iter().enumerate(){
                    // draw the terminals... somehow ...
                    let size = Vec2::splat(2.0 * 4.0);
                    let point_rect = Rect::from_center_size(*t, size);
                    //32 isn't magic, just needs to be larger than the largest number of terminals
                    //that any node can have
                    let point_id = response.id.with(i*32 + j);
                    let point_response = ui.interact(point_rect, point_id, Sense::drag());

                    point_response.context_menu(|ui| {
                        if ui.button("Disconnect terminal").clicked() {
                            ui.close_menu();
                        }
                    });

                    if point_response.is_pointer_button_down_on() {
                        edging |= true;
                        self.drag_start = Some(tpos[j]);
                        if self.drag_target.is_none(){
                            self.drag_target = Some(tpos[j]);
                        }
                        //start_term = Some(t);
                        self.drag_target = Some(self.drag_target.unwrap() + point_response.drag_delta());
                    }
                    else if point_response.drag_stopped(){
                        stopped_edging = true;
                    }
                    else if point_response.hovered(){
                        //target_term = Some(t);
                    }
                }

            }
            if stopped_edging /*&& target_term.is_some()*/{
                //EDGE MATRIX SHIT
            }
            if edging { //draw temp edge as we drag
                painter.line_segment([self.drag_start.unwrap(), self.drag_target.unwrap()],
                                     *EDGE_STROKE);
            }
            else { //if we're done dragging, clear the temp edge state
                self.drag_start = None;
                self.drag_target = None;
            }

            for (_,n) in &self.nodes{

            }

            response
        });
    }
}