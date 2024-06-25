use std::cmp::{max, PartialEq};
use std::iter::repeat;
use std::rc::Rc;
use std::sync::Arc;
use eframe::emath::{Pos2, Rect, vec2, Vec2};
use eframe::epaint::Shape;
use egui::{lerp, Painter, pos2};
use env_logger::Target;
use itertools::Itertools;
use lazy_static::lazy_static;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use petgraph::graph::{Node, NodeIndex};

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
slotmap::new_key_type! { pub struct BlockId; }

#[derive(Clone, Debug, PartialEq)]
pub enum TermType {
    IN,
    OUT
}

#[derive(Clone, Debug, PartialEq, Copy, )]
pub enum BlockType {
    SOURCE(SourceType),
    EFFECT(EffectType),//effect is a bad name
    TARGET(TargetType)
}
#[derive(Clone, Debug, PartialEq, Copy, )]
pub enum SourceType{
    AUDIO,
    CONSTANT,
}
#[derive(Clone, Debug, PartialEq, Copy, EnumIter)]
pub enum EffectType{
    //arithmetic
    ADD, //x+y
    SUB, //x-y
    MUL, //x*y
    DIV, //x/y
    MOD, //x%y
    NEG, //-x
    INV, //1/x
    //trig
    SIN,
    COS,
    TAN,
    COT,
    SEC,
    CSC,
    //coordinates
    P2C, //poltocar
    C2P, //cartopol
    S2C, //sphertocar
    C2S, //cartospher
}
#[derive(Clone, Debug, PartialEq, Copy, )]
pub enum TargetType{
    ITERATOR,
    DISPLAY,
}

fn cat_map(s: &'static str) -> (egui::Color32) {
    match s{
        "Sources" => (egui::Color32::from_rgb(200,190,215)),
        "Arithmetic" => (egui::Color32::from_rgb(225,170,170)),
        "Trig" => (egui::Color32::from_rgb(170,200,150)),
        "Coordinates" => (egui::Color32::from_rgb(170,190,225)),
        "Targets" => (egui::Color32::from_rgb(220,225,180)),
        _ => panic!("category {s} not have entry in cat_map")
    }
}

pub struct BlockArchetype {
    pub name: &'static str,
    pub category: &'static str,
    pub color: egui::Color32,
    pub block_type: BlockType,
    pub inputs: Vec<&'static str>,
    pub outputs: Vec<&'static str>,
    pub size: (f32, f32)
}

impl BlockArchetype {
    fn new(block_type: BlockType,
           name: &'static str,
           category: &'static str,
           inputs: Vec<&'static str>,
           outputs: Vec<&'static str>) -> Self
    {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        match block_type {
            BlockType::SOURCE(_) | BlockType::TARGET(_) => {
                width = 80.0;
                height = max(inputs.len(), outputs.len()) as f32 * 20.0;
            }
            BlockType::EFFECT(_) => {
                width = name.len() as f32 * 10.0;
                //space for terminals +20 for label
                height = max(inputs.len(), outputs.len()) as f32 * 20.0 + 20.0;
            }
        }
        Self {
            name,
            category,
            color: cat_map(category),
            block_type,
            inputs,
            outputs,
            size: (width, height),
        }
    }

    pub fn from_type(n: BlockType) -> Self {
        match n {
        BlockType::SOURCE(st) => {
            let category = "Sources";
            match st {
            SourceType::AUDIO => Self::new(n, "Audio", category, vec![], vec!["RMS", "Pitch"]),
            SourceType::CONSTANT => Self::new(n, "Constant", category, vec![], vec!["0"]),
        }   }
        BlockType::EFFECT(st) => {
            match st {
            EffectType::ADD => Self::new(n, "x+y", "Arithmetic", vec!["x", "y"], vec![""]),
            EffectType::SUB => Self::new(n, "x-y", "Arithmetic", vec!["x", "y"], vec![""]),
            EffectType::MUL => Self::new(n, "x*y", "Arithmetic", vec!["x", "y"], vec![""]),
            EffectType::DIV => Self::new(n, "x/y", "Arithmetic", vec!["x", "y"], vec![""]),
            EffectType::MOD => Self::new(n, "x%y", "Arithmetic", vec!["x", "y"], vec![""]),
            EffectType::NEG => Self::new(n, "-x", "Arithmetic", vec![""], vec![""]),
            EffectType::INV => Self::new(n, "1/x", "Arithmetic", vec![""], vec![""]),
            EffectType::SIN => Self::new(n, "sin(x)", "Trig", vec![""], vec![""]),
            EffectType::COS => Self::new(n, "cos(x)", "Trig", vec![""], vec![""]),
            EffectType::TAN => Self::new(n, "tan(x)", "Trig", vec![""], vec![""]),
            EffectType::COT => Self::new(n, "cot(x)", "Trig", vec![""], vec![""]),
            EffectType::SEC => Self::new(n, "sec(x)", "Trig", vec![""], vec![""]),
            EffectType::CSC => Self::new(n, "csc(x)", "Trig", vec![""], vec![""]),
            EffectType::P2C => Self::new(n, "poltocar", "Coordinates", vec!["r", "θ"], vec!["x", "y"]),
            EffectType::C2P => Self::new(n, "cartopol", "Coordinates", vec!["r", "θ"], vec!["x", "y"]),
            EffectType::S2C =>
                Self::new(n, "sphertocar", "Coordinates", vec!["ρ", "θ", "φ"], vec!["x", "y", "z"]),
            EffectType::C2S =>
                Self::new(n, "cartospher", "Coordinates", vec!["x", "y", "z"], vec!["ρ", "θ", "φ"]),
        }   }
        BlockType::TARGET(st) => {
            match st {
            TargetType::ITERATOR => Self::new(n, "Iterator", "Targets", vec![""], vec![]),
            TargetType::DISPLAY => Self::new(n, "Display", "Targets", vec![""], vec![]),
        }   }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub name: String,
    pub block_type: BlockType,
    pub in_idx: Vec<NodeIndex>,
    pub out_idx: Vec<NodeIndex>,
    //parameters of some kind?
    pub pos: Pos2,
    size: Vec2,
    pub label_color: egui::Color32,
    pub body_rect: Rect,
    label_rect: Rect,
}


impl Block {
    pub fn new(archetype: &BlockArchetype) -> Self{
        Self{
            name: archetype.name.to_owned(),
            block_type: archetype.block_type,
            in_idx: Vec::new(),
            out_idx: Vec::new(),

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
        self.label_rect = match self.block_type {
            BlockType::SOURCE(_) => {
                Rect::from_two_pos(self.body_rect.left_top() + vec2(20.0, 0.0),
                                   self.body_rect.left_bottom())
            }
            BlockType::EFFECT(_) => {
                Rect::from_two_pos(self.body_rect.left_bottom() - vec2(0.0, 20.0),
                                   self.body_rect.right_bottom())
            }
            BlockType::TARGET(_) => {
                Rect::from_two_pos(self.body_rect.right_top() - vec2(20.0, 0.0),
                                   self.body_rect.right_bottom())
            }
        };
    }

    //TODO, add icons or something
    pub fn draw(&self, painter: &Painter){
        painter.rect(self.body_rect, egui::Rounding::ZERO, BODY_COLOR, *BODY_STROKE);
        painter.rect(self.label_rect, egui::Rounding::ZERO, self.label_color, *LABEL_STROKE);

        if matches!(self.block_type, BlockType::EFFECT(_)) {
            painter.text(self.label_rect.center(),
                         egui::Align2::CENTER_CENTER,
                         self.name.clone(),
                         egui::FontId::monospace(11.0),
                         egui::Color32::WHITE);
        }
    }

    pub fn get_terminal_pos(&mut self) -> Vec<(Pos2,NodeIndex)> {
        let in_start = self.body_rect.left_top();
        let in_stop = match self.block_type {
            BlockType::SOURCE(_) => { Pos2::default() }
            BlockType::EFFECT(_) => { self.label_rect.left_top() }
            BlockType::TARGET(_) => { self.body_rect.left_bottom() }
        };
        let in_dist:f32 = (in_stop.y - in_start.y) / (self.in_idx.len() as f32 + 1.0);

        let out_start = self.body_rect.right_top();
        let out_stop = match self.block_type {
            BlockType::SOURCE(_) => { self.body_rect.right_bottom() }
            BlockType::EFFECT(_) => { self.label_rect.left_top() }
            BlockType::TARGET(_) => { Pos2::default() }
        };
        let out_dist:f32 = (out_stop.y - out_start.y) / (self.out_idx.len() as f32 + 1.0);

        (1..=self.in_idx.len()).map(|i|{ in_start + vec2(0.0, i as f32 * in_dist) })
            .chain((1..=self.out_idx.len()).map(|i|{
                out_start + vec2(0.0, i as f32 * out_dist)
            })).zip_eq(self.in_idx.iter().cloned().chain(self.out_idx.iter().cloned())).collect()
    }
}
#[derive(Clone, Debug)]
pub struct Terminal {
    pub pos: Pos2,
    pub name: &'static str,
    pub io: TermType,
}

impl Terminal{
    pub(crate) fn new(name: &'static str, io: TermType) -> Self{
        Self{
            pos: Pos2::default(),
            name,
            io,
        }
    }

    pub fn draw(&self, painter: &Painter, pos: Pos2){
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