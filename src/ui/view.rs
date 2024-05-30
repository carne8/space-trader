use iced::{mouse, widget, Color, Point, Renderer, Size, Theme};
use iced::widget::canvas::{Fill, Path, Style};

use super::types::*;

impl widget::canvas::Program<Message> for App {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<<Renderer as widget::canvas::Renderer>::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

            for system in &self.galaxy {
                let zoomed_system = system.apply_scale(self.zoom.scale);
                let point = Point {
                    x: zoomed_system.x + self.zoom.offset.x + self.nav.offset.x,
                    y: zoomed_system.y + self.zoom.offset.y + self.nav.offset.y,
                };

                if !bounds.expand(zoomed_system.size).contains(point) {
                    continue;
                }

                if system.distance > 0.16 && self.zoom.scale > 6.5 {
                    frame.fill(
                        &Path::circle(point, zoomed_system.size),
                        Fill {
                            style: Style::Solid(zoomed_system.color),
                            ..Fill::default()
                        },
                    );

                    for w in zoomed_system.waypoints {
                        frame.fill_rectangle(
                            Point::new(point.x + w.x - w.size / 2., point.y + w.y - w.size / 2.),
                            Size::new(w.size, w.size),
                            Color::WHITE,
                        )
                    }
                } else {
                    frame.fill_rectangle(point, Size::new(2., 2.), zoomed_system.color)
                }
            }
        });

        vec![geometry]
    }
}
