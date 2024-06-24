use egui::epaint::{PathShape};
use egui::*;
use itertools::Itertools;
use crate::editors::response_curve_editor::Curve::Overall;

#[derive(Debug, PartialEq)]
enum Curve{
    Overall,
    Red,
    Green,
    Blue,
    Alpha
}
pub struct ResponseCurveEditor{
    selected_curve: Curve,

    /// The control points.
    points_o: Vec<Pos2>,
    points_r: Vec<Pos2>,
    points_g: Vec<Pos2>,
    points_b: Vec<Pos2>,
    points_a: Vec<Pos2>,

    /// Stroke for auxiliary line.
    stroke_o: Stroke,
    stroke_r: Stroke,
    stroke_g: Stroke,
    stroke_b: Stroke,
    stroke_a: Stroke,

    n_points: usize,
    selected_index: usize,

    bounding_box_stroke: Stroke,
}

///TODO
/// This editor should allow you to switch between five different value curves (rgba+overall).
/// We would like to display a histogram underneath the curve, showing the distribution of tones in the image.
impl Default for ResponseCurveEditor {
    fn default() -> Self {
        let default_curve = vec![pos2(0.0, 1.0),
                                            pos2(0.25,0.75),
                                            pos2(0.5,0.5),
                                            pos2(0.75,0.25),
                                            pos2(1.0,0.0)];
        Self {
            selected_curve: Overall,
            points_o: default_curve.clone(), //invert Y value when calculating actual value-curve
            points_r: default_curve.clone(),
            points_g: default_curve.clone(),
            points_b: default_curve.clone(),
            points_a: default_curve.clone(),
            stroke_o: Stroke::new(2.0, Color32::WHITE.linear_multiply(0.25)),
            stroke_r: Stroke::new(2.0, Color32::RED.linear_multiply(0.25)),
            stroke_g: Stroke::new(2.0, Color32::GREEN.linear_multiply(0.25)),
            stroke_b: Stroke::new(2.0, Color32::BLUE.linear_multiply(0.25)),
            stroke_a: Stroke::new(2.0, Color32::GRAY.linear_multiply(0.25)),
            selected_index: 0,
            n_points: 5,
            bounding_box_stroke: Stroke::new(1.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        }
    }
}

//TODO--SHOULD NOT BE POSSIBLE TO MAKE A GRAPH THAT IS NOT A FUNCTION, SORT POINTS BY X VALUE?
//TODO--CLICK ANYWHERE ON THE LINE TO ADD ANOTHER POINT
impl ResponseCurveEditor {
    pub fn ui_content(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_curve, Curve::Overall, "Overall");
                ui.selectable_value(&mut self.selected_curve, Curve::Red, "Red");
                ui.selectable_value(&mut self.selected_curve, Curve::Green, "Green");
                ui.selectable_value(&mut self.selected_curve, Curve::Blue, "Blue");
                ui.selectable_value(&mut self.selected_curve, Curve::Alpha, "Alpha");
            });
            let (response, painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height()),
                                    Sense::drag().union(Sense::click()));

            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, vec2(1.0, 1.0)),
                response.rect,
            );
            let xwidth = response.rect.max[0] - response.rect.min[0];
            let ywidth = response.rect.max[1] - response.rect.min[1];
            let scale: Vec2 = vec2(1.0 / xwidth, 1.0 / ywidth);

            let control_point_radius = 6.0;
            let selected_points = match self.selected_curve {
                Curve::Overall => &mut self.points_o,
                Curve::Red => &mut self.points_r,
                Curve::Green => &mut self.points_g,
                Curve::Blue => &mut self.points_b,
                Curve::Alpha => &mut self.points_a,
            };
            let control_point_shapes: Vec<Shape> = selected_points
                .iter_mut()
                .enumerate()
                .map(|(i, point)| {
                    let size = Vec2::splat(2.0 * control_point_radius);

                    let point_in_screen = to_screen.transform_pos(*point);
                    let point_rect = Rect::from_center_size(point_in_screen, size);
                    let point_id = response.id.with(i);
                    let point_response = ui.interact(point_rect, point_id, Sense::drag());

                    let mut new_point = *point + point_response.drag_delta() * scale;
                    new_point = to_screen.from().clamp(new_point);

                    if i == 0 {
                        new_point.x = 0.0; // Keep the first point on the left edge
                    } else if i == self.n_points - 1 {
                        new_point.x = 1.0; // Keep the last point on the right edge
                    }

                    *point = new_point;

                    let point_in_screen = to_screen.transform_pos(*point);
                    let stroke = ui.style().interact(&point_response).fg_stroke;

                    Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
                })
                .collect();

            let line_click_radius = 8.0;
            let mut new_point = None;

            if response.secondary_clicked() {
                let click_pos = response.interact_pointer_pos().unwrap();
                let click_pos_transformed = to_screen.inverse().transform_pos(click_pos);

                println!("Right-click detected at position: {:?}", click_pos_transformed);

                for (i, point) in selected_points.iter().enumerate() {
                    println!("Checking point {}: {:?}", i, point);

                    if point.distance(click_pos_transformed) < control_point_radius * scale.x {
                        println!("Point {} is within click radius", i);

                        if i > 0 && i < selected_points.len() - 1 {
                            println!("Removing point {}", i);
                            selected_points.remove(i);
                        } else {
                            println!("Cannot remove point {}: it's an endpoint", i);
                        }
                        break;
                    }
                }
            }

            let points_in_screen: Vec<Pos2> = selected_points.iter()
                .sorted_by(|a, b| a.x.partial_cmp(&b.x).unwrap())
                .map(|p| to_screen * *p)
                .collect();


            if response.clicked() {
                let click_pos = response.interact_pointer_pos().unwrap();
                let click_pos_transformed = to_screen.inverse().transform_pos(click_pos);

                for i in 0..points_in_screen.len() - 1 {
                    let p1 = points_in_screen[i];
                    let p2 = points_in_screen[i + 1];

                    if let Some(closest_point) = closest_point_on_line_segment(click_pos, p1, p2) {
                        if closest_point.distance(click_pos) < line_click_radius {
                            new_point = Some(closest_point);
                            break;
                        }
                    }
                }

                if let Some(new_point) = new_point {
                    selected_points.push(click_pos_transformed);
                }
            }

            painter.add(PathShape::line(points_in_screen, *match self.selected_curve {
                Curve::Overall => &self.stroke_o,
                Curve::Red => &self.stroke_r,
                Curve::Green => &self.stroke_g,
                Curve::Blue => &self.stroke_b,
                Curve::Alpha => &self.stroke_a,
            }));
            painter.extend(control_point_shapes);
        });
    }
}

fn closest_point_on_line_segment(point: Pos2, p1: Pos2, p2: Pos2) -> Option<Pos2> {
    let line_segment = p2 - p1;
    let point_vector = point - p1;

    let dot_product = line_segment.x * point_vector.x + line_segment.y * point_vector.y;
    let line_segment_length_squared = line_segment.x * line_segment.x + line_segment.y * line_segment.y;

    if line_segment_length_squared == 0.0 {
        return None;
    }

    let t = dot_product / line_segment_length_squared;

    if t < 0.0 || t > 1.0 {
        return None;
    }

    Some(p1 + line_segment * t)
}