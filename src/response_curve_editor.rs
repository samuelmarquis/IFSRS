use egui::epaint::{CubicBezierShape, PathShape, QuadraticBezierShape};
use egui::*;
use crate::response_curve_editor::Curve::Overall;

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
    overall_points: Vec<Pos2>,
    red_points: Vec<Pos2>,
    green_points: Vec<Pos2>,
    blue_points: Vec<Pos2>,
    alpha_points: Vec<Pos2>,

    /// Stroke for auxiliary line.
    stroke: Stroke,

    bounding_box_stroke: Stroke,
}

impl Default for ResponseCurveEditor {
    fn default() -> Self {
        let default_curve = vec![pos2(0.0, 1.0),
                                            pos2(0.25,0.75),
                                            pos2(0.5,0.5),
                                            pos2(0.75,0.25),
                                            pos2(1.0,0.0)];
        Self {
            selected_curve: Overall,
            overall_points: default_curve.clone(), //invert Y value after
            red_points: default_curve.clone(),
            green_points: default_curve.clone(),
            blue_points: default_curve.clone(),
            alpha_points: default_curve.clone(),
            stroke: Stroke::new(3.0, Color32::RED.linear_multiply(0.25)),
            bounding_box_stroke: Stroke::new(1.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        }
    }
}

//TODO--SHOULD NOT BE POSSIBLE TO MAKE A GRAPH THAT IS NOT A FUNCTION, SORT POINTS BY X VALUE?
//TODO--CLICK ANYWHERE ON THE LINE TO ADD ANOTHER POINT
impl ResponseCurveEditor {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        ui.horizontal(|ui|{
            ui.selectable_value(&mut self.selected_curve, Curve::Overall, "Overall");
            ui.selectable_value(&mut self.selected_curve, Curve::Red, "Red");
            ui.selectable_value(&mut self.selected_curve, Curve::Green, "Green");
            ui.selectable_value(&mut self.selected_curve, Curve::Blue, "Blue");
            ui.selectable_value(&mut self.selected_curve, Curve::Alpha, "Alpha");
        });
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, vec2(1.0,1.0)),
            response.rect,
        );
        let xwidth = response.rect.max[0] - response.rect.min[0];
        let ywidth = response.rect.max[1] - response.rect.min[1];
        let scale : Vec2 = vec2(1.0/xwidth, 1.0/ywidth);

        let control_point_radius = 8.0;

        let control_point_shapes: Vec<Shape> = self
            .overall_points
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                *point += point_response.drag_delta() * scale;
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

        //let mut all_points : [Pos2; self.control_points.len()+2] = self.control_points;

        let points_in_screen: Vec<Pos2> = self
            .overall_points
            .iter()
            .map(|p| to_screen * *p)
            .collect();

        painter.add(PathShape::line(points_in_screen, self.stroke));
        painter.extend(control_point_shapes);

        response
    }
}