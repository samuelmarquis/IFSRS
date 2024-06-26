use std::iter::*;
use std::vec;

use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, pos2, Sense};
use itertools::Itertools;
use lazy_static::lazy_static;
use petgraph::prelude::*;
use petgraph::visit::{IntoEdgeReferences, IntoEdges};
use slotmap::HopSlotMap;
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
    graph: StableGraph<Terminal, bool, Undirected>,
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
            once(BlockArchetype::from_type(BlockType::SOURCE(SourceType::AUDIO)))
            .chain(once(BlockArchetype::from_type(BlockType::SOURCE(SourceType::CONSTANT))))
            .chain(EffectType::iter().map(|f| { BlockArchetype::from_type(BlockType::EFFECT(f))}))
            .chain(once(BlockArchetype::from_type(BlockType::TARGET(TargetType::DISPLAY))))
            .collect();
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

    /// Draws the window
    /// Draws blocks & terminals
    ///Enables:
    /// Adding, removing, & repositioning blocks
    /// Creating & deleting connections between terminals
    /// The editing of block properties via the side-panel
    ///Ensures:
    /// only valid connections are made
    /// TODO--drag from IN-terminal with connection to drag the OUT's connection somewhere else
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
                    ui.label("Select a node to see properties");
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
            if response.secondary_clicked() { //steals clicks from the context menu?
                self.click_pos = ui.ctx().pointer_interact_pos();
            }

            response.context_menu(|ui| {
                let mut categories = self.archetypes.iter()
                    .map(|a| a.category)
                    .collect::<Vec<_>>();
                categories.dedup(); //NOTE--CATEGORIES MUST BE ALREADY SORTED

                for category in categories { ui.menu_button(category,|ui| {
                    for archetype in self.archetypes.iter()
                        .filter(|a|a.category == category)
                    { if ui.button(archetype.name).clicked() {
                        if let Some(pos) = self.click_pos {
                            AutomationEditor::add_block(&mut self.blocks, &mut self.graph, archetype, pos);
                            ui.close_menu();
                        }
                    } }
                });}
            });

            let mut edging: bool = false;
            let mut stopped_edging: bool = false;
            let mut prune_term: Option<NodeIndex> = None;
            let mut delete_block: Option<BlockId> = None;
            let mut process_queue: Vec<NodeIndex> = Vec::new();

            //draw edges first so they go under nodes
            for e in self.graph.edge_references(){
                painter.line_segment([self.graph[e.source()].pos, self.graph[e.target()].pos],
                                     *EDGE_STROKE);
            }

            for (id, block) in self.blocks.iter_mut(){
                let node_id = response.id.with(id);
                let node_response = ui.interact(block.body_rect, node_id, Sense::drag());
                if node_response.drag_delta().length() > 0.0 {
                    block.update(Some(block.pos + node_response.drag_delta()));
                    //todo: only update terminal position here
                }
                if node_response.is_pointer_button_down_on(){
                    self.selected_block = Some(id);
                }

                node_response.context_menu(|ui| {
                    if ui.button("Delete Node").clicked() {
                        delete_block = Some(id);
                        ui.close_menu();
                    }
                });

                if matches!(block.block_type, BlockType::TARGET(TargetType::DISPLAY))
                && block.val == None { //todo check hash & timestep
                    process_queue.push(*block.in_idx.first().unwrap())
                }
                block.draw(&painter);

                for (p, t) in block.get_terminals()
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

            for id in process_queue{
                self.process(id);
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
            if let Some(block) = delete_block {
                //refactor if we ever decouple opening the block context menu from selecting
                self.selected_block = None;
                for (_,t) in self.blocks[block].get_terminals(){
                    let d:Vec<EdgeIndex> = self.graph.edges(t).map(|e|e.id()).collect();
                    for a in d{
                        self.graph.remove_edge(a);
                    }
                    self.graph.remove_node(t);
                }
                self.blocks.remove(block);
            }
            response
        });
    }

    fn add_block<'a>(
        blocks: & mut Blocks,
        graph: &mut StableGraph<Terminal, bool, Undirected>,
        arch: &BlockArchetype,
        pos: Pos2) -> BlockId
    {
        let id = blocks.insert(Block::new(arch));
        blocks[id].in_idx = arch.inputs
            .iter()
            .map(|s| { graph.add_node(Terminal::new(s, TermType::IN, id)) })
            .collect();
        blocks[id].out_idx = arch.outputs
            .iter()
            .map(|s| { graph.add_node(Terminal::new(s, TermType::OUT, id)) })
            .collect();
        blocks[id].update(Some(pos));
        return id
    }
    //TODO--Enable removing terminals
    fn update_block(
        b: &mut Block,
        graph: &mut StableGraph<Terminal, bool, Undirected>,
        term: Terminal,
        name: Option<String>)
    {
        b.in_idx.push(graph.add_node(term));
        b.update(None);
        if let Some(s) = name {
            b.name = s; //eugh
        }
    }

    /// Given an iterator id, iterator name, and parameter name,
    /// looks for a target block with that iterator id and adds the new param,
    /// updating the name if necessary.
    /// If the target isn't found, it creates one with the param.
    /// In either case we return a NodeIndex, so that the renderer can call process() with it
    /// to get a value for the automated field.
    pub fn update_target(
        &mut self,
        it_id: i32,
        it_name: String,
        param_name: String,) -> NodeIndex
    {
        //Iterator block exists, find the block and update it
        if let Some((id, b)) = self.blocks
            .iter_mut()
            .find(|(id, b)|
                {matches!(b.block_type, BlockType::TARGET(TargetType::ITERATOR(it_id)))})
        {
            AutomationEditor::update_block(
                b,
                &mut self.graph,
                Terminal::new(param_name.leak(), TermType::IN, id),
                Some(it_name));
            // 🤓 umm did you know that return statements are optional in rust 🤓
            return b.in_idx.last().unwrap().clone();
        }
        //Iterator block doesn't exist, create it
        else {
            let arch = BlockArchetype::new(
                BlockType::TARGET(TargetType::ITERATOR(it_id)),
                it_name.leak(),
                "Iterators",
                vec![param_name.leak()],
                vec![]);
            let id = AutomationEditor::add_block(
                &mut self.blocks,
                &mut self.graph,
                &arch,
                pos2(350.0,50.0));
            return self.blocks[id].in_idx.last().unwrap().clone();
        }
    }

    /// Computes a value from one target terminal at the current timestep.
    /// Walks backwards through the graph, calculating & storing values as it does so.
    pub fn process(&mut self, id: NodeIndex){

        if let Some(idx) = self.graph.neighbors(id).next() {

            let b_id = self.graph[idx].owner;
            let n_idx : Vec<NodeIndex> = self.blocks[b_id].in_idx.iter().cloned().collect();
            for b in n_idx{
                self.process(b)
            }
        }
        else{
            //value is uncomputable if node is disconnected
        }
    }
}