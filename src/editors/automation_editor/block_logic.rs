use nalgebra::RealField;
use crate::editors::automation_editor::blocks::*;

fn square(f: f32) -> f32 {
  f * f
}

pub fn effect_logic(st: EffectType, args: Vec<f32>) -> Vec<f32> {
  match st {
    EffectType::ADD => vec![args[0] + args[1]],
    EffectType::SUB => vec![args[0] - args[1]],
    EffectType::MUL => vec![args[0] * args[1]],
    EffectType::DIV => vec![args[0] / args[1]],
    EffectType::MOD => vec![args[0] % args[1]],
    EffectType::NEG => vec![-args[0]],
    EffectType::INV => vec![1.0 / args[0]],
    EffectType::SIN => vec![f32::sin(args[0])],
    EffectType::COS => vec![f32::cos(args[0])],
    EffectType::TAN => vec![f32::tan(args[0])],
    EffectType::COT => vec![-f32::tan(args[0] + f32::frac_pi_2())],
    EffectType::SEC => vec![f32::tan(args[0]) / f32::sin(args[0])],
    EffectType::CSC => vec![-f32::tan(args[0] + f32::frac_pi_2()) / f32::cos(args[0])],
    EffectType::P2C => vec![args[0] * f32::cos(args[1]), args[0] * f32::sin(args[1])],
    EffectType::C2P => vec![f32::sqrt(square(args[0]) + square(args[1])), //r
                            f32::atan(args[1] / args[0])], //θ
    EffectType::S2C => vec![args[0] * f32::sin(args[1]) * f32::cos(args[2]),
                            args[0] * f32::sin(args[1]) * f32::sin(args[2]),
                            args[0] * f32::cos(args[1])],
    EffectType::C2S => {
      let r = f32::sqrt(square(args[0]) + square(args[1]) + square(args[2]));
      let r_d = f32::sqrt(square(args[0]) + square(args[1]));
      vec![r,
           f32::acos(args[2] / r),
           f32::signum(args[1]) * f32::acos(args[1]) / r_d]
    }
  }
}
