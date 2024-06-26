
use std::cmp::{max, PartialEq};
use std::collections::HashMap;
use std::iter::*;
use std::rc::Rc;
use std::{iter, vec};
use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Painter, Sense, Ui};
use itertools::Itertools;
use lazy_static::lazy_static;
use nalgebra::point;
use slotmap::HopSlotMap;
use petgraph::prelude::*;
use petgraph::visit::{IntoEdgeReferences, IntoEdges};
use strum::IntoEnumIterator;
use crate::editors::concepts::blocks::*;

const EDGE_COLOR: egui::Color32 = egui::Color32::from_rgb(255,255,255);
lazy_static! {
    static ref EDGE_STROKE: egui::Stroke = egui::Stroke::new(2.0, EDGE_COLOR);
}

pub type Blocks = HopSlotMap<BlockId, Block>;

pub struct AutomationEditor {
    anim_frame: usize, // TODO -- GET THIS FROM THE APP
    archetypes: Vec<BlockArchetype>,
    blocks: Blocks,
    //the graph is undirected because parallel edges do not make sense in this context
    graph: StableGraph<Terminal, bool, petgraph::Undirected>,

    selected_block: Option<BlockId>,
    click_pos: Option<Pos2>,

    drag_start: Option<Pos2>,
    drag_target: Option<Pos2>,

    term_start: Option<NodeIndex>,
    term_target: Option<NodeIndex>,
}

impl Default for AutomationEditor{
    fn default() -> Self {
        let archetypes =
            iter::once(BlockArchetype::from_type(BlockType::SOURCE(SourceType::AUDIO))).chain(
            iter::once(BlockArchetype::from_type(BlockType::SOURCE(SourceType::CONSTANT)))).chain(
            iter::once(BlockArchetype::from_type(BlockType::TARGET(TargetType::DISPLAY)))).chain(
            EffectType::iter().map(|f| {
            BlockArchetype::from_type(BlockType::EFFECT(f))
            })).collect();
        Self {
            anim_frame: 0,
            archetypes: archetypes,
            blocks: Blocks::default(),
            //terminals: Terminals::default(),
            graph: StableGraph::default(),
            selected_block: None,
            click_pos: None,
            drag_start: None,
            drag_target: None,
            term_start: None,
            term_target: None,
        }
    }
}

impl AutomationEditor {
    pub fn ui_content(&mut self, ctx: &Context) {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(n_id) = self.selected_block {
                    let n = &self.blocks[n_id];
                    ui.label(n.name.clone());
                    ui.separator();
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
                categories.dedup(); //NOTE--CATEGORIES MUST BE ALREADY SORTED

                for category in categories { ui.menu_button(category,|ui| {
                    for archetype in self.archetypes.iter()
                        .filter(|x|x.category == category)
                    { if ui.button(archetype.name).clicked() {
                        if let Some(pos) = self.click_pos {
                            let x = self.blocks.insert(Block::new(archetype));
                            self.blocks[x].set_pos(pos);
                            self.blocks[x].in_idx = (archetype.inputs.iter().map(|s|
                            { self.graph.add_node(Terminal::new(s, TermType::IN)) }))
                                .collect();
                            self.blocks[x].out_idx = (archetype.outputs.iter().map(|s|
                            { self.graph.add_node(Terminal::new(s, TermType::OUT)) }))
                                .collect();
                            ui.close_menu();
                        }
                    } }
                });}
            });

            let mut edging: bool = false;
            let mut stopped_edging: bool = false;
            let mut prune_term: Option<NodeIndex> = None;
            let mut delete_node: Option<BlockId> = None;

            //draw edges first so they go under nodes
            for e in self.graph.edge_references(){
                painter.line_segment([self.graph[e.source()].pos, self.graph[e.target()].pos],
                                     *EDGE_STROKE);
            }

            for (i, n) in self.blocks.iter_mut(){
                let node_id = response.id.with(i);
                let node_response = ui.interact(n.body_rect, node_id, Sense::drag());
                n.set_pos(n.pos + node_response.drag_delta());
                if node_response.is_pointer_button_down_on(){
                    self.selected_block = Some(i);
                }

                node_response.context_menu(|ui| {
                    if ui.button("Delete Node").clicked() {
                        delete_node = Some(i);
                        ui.close_menu();
                    }
                });

                n.draw(&painter);

                for (p, t) in n.get_terminal_pos()
                {

                    self.graph[t].pos = p;
                    let term = &self.graph[t];
                    // Set click region
                    term.draw(&painter, p);
                    let term_size = Vec2::splat(2.0 * 4.0);
                    let term_rect = Rect::from_center_size(p, term_size);
                    //32 isn't magic, just needs to be larger than the largest number of terminals
                    //that any node can have
                    let term_id = response.id.with(t);
                    let term_response = ui.interact(term_rect, term_id, Sense::drag());

                    // Click handling
                    term_response.context_menu(|ui| {
                        if ui.button("Disconnect terminal").clicked() {
                            prune_term = Some(t);
                            ui.close_menu();
                        }
                    });
                    if term_response.is_pointer_button_down_on()
                    && term.io == TermType::OUT //drags can only start from OUT
                    {
                        edging |= true;

                        if self.drag_target.is_none(){
                            self.drag_start = Some(p);
                            self.drag_target = Some(p);
                            self.term_start = Some(t);
                        }

                        self.drag_target = Some(self.drag_target.unwrap() + term_response.drag_delta());
                    }
                    else if term_response.drag_stopped(){
                        stopped_edging = true;
                    }
                    else if term_response.hovered(){
                        self.term_target = Some(t);
                    }
                }
            }

            if stopped_edging {
                if let (Some(a), Some(b)) = (self.term_start, self.term_target){
                    if self.graph[b].io == TermType::IN { //drag always starts at OUT
                        let d:Vec<EdgeIndex> = self.graph.edges(b).map(|e|e.id()).collect();
                        for e in d{
                            self.graph.remove_edge(e);
                        }
                        self.graph.add_edge(a, b, true);
                    }
                }
            }
            if edging { //draw temp edge as we drag
                painter.line_segment([self.drag_start.unwrap(), self.drag_target.unwrap()],
                                     *EDGE_STROKE);
            }
            else { //if we're done dragging, clear the temp edge state
                self.drag_start = None;
                self.drag_target = None;
                self.term_start = None;
                self.term_target = None;
            }

            if let Some(t) = prune_term {
                let d:Vec<EdgeIndex> = self.graph.edges(t).map(|e|e.id()).collect();
                for a in d{
                    self.graph.remove_edge(a);
                }
            }
            if let Some(n) = delete_node {
                self.selected_block = None; //be more granular, blanket solution
                for (_,t) in self.blocks[n].get_terminal_pos(){
                    let d:Vec<EdgeIndex> = self.graph.edges(t).map(|e|e.id()).collect();
                    for a in d{
                        self.graph.remove_edge(a);
                    }
                    self.graph.remove_node(t);
                }
                self.blocks.remove(n);
            }

            response
        });
    }
}