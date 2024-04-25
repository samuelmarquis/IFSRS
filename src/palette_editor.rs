use eframe::emath;
use eframe::emath::{Pos2, pos2, Rect, Vec2, vec2};
use eframe::epaint::{Color32, PathShape, Shape, Stroke};
use egui::{Sense, Ui};
use crate::response_curve_editor::ResponseCurveEditor;

pub struct PaletteEditor{

}

impl Default for PaletteEditor {
    fn default() -> Self {
        Self {

        }
    }
}
/*
impl PaletteEditor {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
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
            .control_points
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
            .control_points
            .iter()
            .map(|p| to_screen * *p)
            .collect();

        painter.add(PathShape::line(points_in_screen, self.stroke));
        painter.extend(control_point_shapes);

        response
    }
}*/