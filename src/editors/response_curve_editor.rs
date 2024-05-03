use egui::epaint::{PathShape};
use egui::*;
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

    bounding_box_stroke: Stroke,
}

///TODO
/// This editor should allow you to switch between five different value curves for each image channel, as well as their sum.
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
            ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height()), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, vec2(1.0,1.0)),
            response.rect,
        );
        let xwidth = response.rect.max[0] - response.rect.min[0];
        let ywidth = response.rect.max[1] - response.rect.min[1];
        let scale : Vec2 = vec2(1.0/xwidth, 1.0/ywidth);

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

                *point += point_response.drag_delta() * scale;
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

        //let mut all_points : [Pos2; self.control_points.len()+2] = self.control_points;

        let points_in_screen: Vec<Pos2> = selected_points
            .iter()
            .map(|p| to_screen * *p)
            .collect();

        painter.add(PathShape::line(points_in_screen, *match self.selected_curve {
            Curve::Overall => &self.stroke_o,
            Curve::Red => &self.stroke_r,
            Curve::Green => &self.stroke_g,
            Curve::Blue => &self.stroke_b,
            Curve::Alpha => &self.stroke_a,
        }));
        painter.extend(control_point_shapes);

        response
    }
}