use std::cmp::{max, PartialEq};
use std::iter::repeat;
use std::rc::Rc;
use std::sync::Arc;
use eframe::emath::{Pos2, Rect, vec2, Vec2};
use eframe::epaint::Shape;
use egui::{lerp, Painter, pos2};
use lazy_static::lazy_static;

const BODY_COLOR: egui::Color32 = egui::Color32::from_rgb(128, 128, 128);
const TERMINAL_COLOR: egui::Color32 = egui::Color32::from_rgb(255,255,255);

const BODY_STROKE_COLOR: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);
const LABEL_STROKE_COLOR: egui::Color32 = egui::Color32::from_rgb(8, 8, 8);
const TERMINAL_STROKE_COLOR: egui::Color32 = egui::Color32::from_rgb(8, 8, 8);

lazy_static! {
    //WHY is this initializer not const??
    static ref BODY_STROKE: egui::Stroke = egui::Stroke::new(1.0, BODY_STROKE_COLOR);
    static ref LABEL_STROKE: egui::Stroke = egui::Stroke::new(1.0, LABEL_STROKE_COLOR);
    static ref TERMINAL_STROKE: egui::Stroke = egui::Stroke::new(1.0, TERMINAL_STROKE_COLOR);

}

slotmap::new_key_type! { pub struct TermId; }
slotmap::new_key_type! { pub struct NodeId; }

#[derive(Clone, Debug)]
pub enum TermType {
    IN,
    OUT
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum NodeType{
    SOURCE,
    EFFECT,
    TARGET
}

pub struct NodeArchetype {
    pub name: &'static str,
    pub category: &'static str,
    pub color: egui::Color32,
    pub node_type: NodeType,
    pub inputs: Vec<&'static str>,
    pub outputs: Vec<&'static str>,
    pub size: (f32, f32)
}

impl NodeArchetype {
    pub fn new(name: &'static str,
               category: (&'static str, egui::Color32, NodeType),
               inputs: Vec<&'static str>,
               outputs: Vec<&'static str>) -> Self
    {
        let mut width : f32 = 0.0;
        let mut height: f32 = 0.0;

        match category.2{
            /*NodeType::SOURCE => {
                //wider for left-label
                width = 80.0;
                height = max(inputs.len(),outputs.len()) as f32 * 20.0
            }*/
            NodeType::EFFECT => {
                width = name.len() as f32 * 10.0;
                //space for terminals +20 for label
                height = max(inputs.len(),outputs.len()) as f32 * 20.0 + 20.0;
            }
            NodeType::TARGET | NodeType::SOURCE => {
                //wider for side-label
                width = 80.0;
                height = max(inputs.len(),outputs.len()) as f32 * 20.0
            }
        }

        Self {
            name: name,
            category: category.0,
            color: category.1,
            node_type: category.2,
            inputs: inputs,
            outputs: outputs,
            size: (width, height)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub ins: usize,
    pub outs: usize,
    //parameters of some kind?
    pub pos: Pos2,
    pub size: Vec2,
    pub label_color: egui::Color32,
    body_rect: Rect,
    label_rect: Rect,
}


impl Node{
    pub fn new(archetype: &NodeArchetype) -> Self{
        Self{
            name: archetype.name.to_owned(),
            node_type: archetype.node_type,
            ins: archetype.inputs.len(),
            outs: archetype.outputs.len(),

            pos: Pos2::default(),
            size: Vec2::from(archetype.size),
            label_color: archetype.color,
            body_rect: Rect::ZERO,
            label_rect: Rect::ZERO,
        }
    }

    pub fn set_pos(&mut self, pos: Pos2){
        self.pos = pos;
        self.body_rect = Rect::from_min_size(self.pos, self.size);
        self.label_rect = match self.node_type{
            NodeType::SOURCE => {
                Rect::from_two_pos(self.body_rect.left_top() + vec2(20.0, 0.0),
                                   self.body_rect.left_bottom())
            }
            NodeType::EFFECT => {
                Rect::from_two_pos(self.body_rect.left_bottom() - vec2(0.0, 20.0),
                                   self.body_rect.right_bottom())
            }
            NodeType::TARGET => {
                Rect::from_two_pos(self.body_rect.right_top() - vec2(20.0, 0.0),
                                   self.body_rect.right_bottom())
            }
        };
    }

    //TODO, add icons or something
    pub fn draw(&self, painter: &Painter){
        painter.rect(self.body_rect, egui::Rounding::ZERO, BODY_COLOR, *BODY_STROKE);
        painter.rect(self.label_rect, egui::Rounding::ZERO, self.label_color, *LABEL_STROKE);

        if self.node_type == NodeType::EFFECT {
            painter.text(self.label_rect.center(),
                         egui::Align2::CENTER_CENTER,
                         self.name.clone(),
                         egui::FontId::monospace(11.0),
                         egui::Color32::WHITE);
        }
    }

    pub fn get_terminal_pos(&self) -> Vec<Pos2> {
        let in_start = self.body_rect.left_top();
        let in_stop = match self.node_type{
            NodeType::SOURCE => { Pos2::default() }
            NodeType::EFFECT => { self.label_rect.left_top() }
            NodeType::TARGET => { self.body_rect.left_bottom() }
        };
        let in_dist:f32 = (in_stop.y - in_start.y) / (self.ins as f32 + 1.0);

        let out_start = self.body_rect.right_top();
        let out_stop = match self.node_type{
            NodeType::SOURCE => { self.body_rect.right_bottom() }
            NodeType::EFFECT => { self.label_rect.left_top() }
            NodeType::TARGET => { Pos2::default() }
        };
        let out_dist:f32 = (out_stop.y - out_start.y) / (self.outs as f32 + 1.0);

        let mut in_pos: Vec<Pos2> = (1..=self.ins).map(|i|{
            in_start + vec2(0.0, i as f32 * in_dist) }).collect();

        let mut out_pos: Vec<Pos2> = (1..=self.outs).map(|i|{
            out_start + vec2(0.0, i as f32 * out_dist) }).collect();

        in_pos.append(&mut out_pos);
        in_pos
    }
}

//nodes possess a bunch of these and they connect to each other. maybe
#[derive(Clone, Debug)]
pub struct Terminal {
    pub name: &'static str,
    pub io: TermType,
}

impl Terminal{
    pub(crate) fn new(name: &'static str, io: TermType) -> Self{
        Self{
            name,
            io,
        }
    }

    fn draw(&self, painter: &Painter, pos: Pos2){
        painter.add(Shape::circle_stroke(pos, 4.0, *TERMINAL_STROKE));
        painter.add(Shape::circle_filled(pos, 3.0, TERMINAL_COLOR));
        match self.io{
            TermType::IN => {
                painter.text(pos + vec2(6.0, 0.0),
                             egui::Align2::LEFT_CENTER,
                             self.name,
                             egui::FontId::monospace(10.0),
                             egui::Color32::WHITE);
            }
            TermType::OUT => {
                painter.text(pos + vec2(-6.0, 0.0),
                             egui::Align2::RIGHT_CENTER,
                             self.name,
                             egui::FontId::monospace(10.0),
                             egui::Color32::WHITE);
            }
        }
    }
}