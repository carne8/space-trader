use iced::*;

#[derive(Default)]
pub struct App {
    cache: widget::canvas::Cache,
    universe: Vec<space_trader_api::models::System>,
    zoom: Zoom,
    nav: Navigation
}

struct Zoom {
    scale: f32,
    offset: Vector
}
impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            scale: 1.,
            offset: Vector::ZERO
        }
    }
}

#[derive(Default)]
struct Navigation {
    offset_start_position: Option<Point>,
    mouse_current_position: Point,
    offset: Vector
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    MouseDown,
    MouseUp,
    MouseMoved(Point),
    MouseWheelScrolled(f32),
}

impl Application for App {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let universe = {
            let text = std::fs::read_to_string("./systems.json").unwrap();
            let systems = serde_json::from_str(&text).unwrap();

            systems
        };

        (
            App {
                universe,
                ..App::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Space Traders")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MouseWheelScrolled(scroll_delta) => {
                let zoom_coef = 0.1;
                let scale = (scroll_delta * zoom_coef).max(-1.).min(1.).exp().max(0.01);

                let mouse = self.nav.mouse_current_position;

                self.zoom = Zoom {
                    scale: self.zoom.scale * scale,
                    offset: Vector::new(
                        self.zoom.offset.x * scale + (mouse.x - mouse.x * scale),
                        self.zoom.offset.y * scale + (mouse.y - mouse.y * scale),
                    ),
                };
                self.cache.clear();
            }
            Message::MouseDown => {
                self.nav.offset_start_position = Some(Point {
                    x: self.nav.mouse_current_position.x - self.nav.offset.x,
                    y: self.nav.mouse_current_position.y - self.nav.offset.y,
                });
            }
            Message::MouseUp => {
                self.nav.offset_start_position = None;
            }
            Message::MouseMoved(position) => {
                self.nav.mouse_current_position = position;

                if let Some(mouse_down_position) = self.nav.offset_start_position {
                    self.nav.offset = position - mouse_down_position;
                    self.cache.clear();
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        widget::Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        event::listen_with(|event, _| match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: _, y },
                }
                | mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: _, y },
                } => Some(Message::MouseWheelScrolled(y)),

                mouse::Event::ButtonPressed(mouse::Button::Left) => Some(Message::MouseDown),
                mouse::Event::ButtonReleased(mouse::Button::Left) => Some(Message::MouseUp),

                mouse::Event::CursorMoved { position } => Some(Message::MouseMoved(position)),
                _ => None,
            },
            _ => None,
        })
    }
}

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

            let width = frame.width().min(frame.height());
            let max = self
                .universe
                .iter()
                .map(|system| system.x.max(system.y))
                .max()
                .unwrap() as f32;

            // The ratio for the systems to be drawn in the center of the screen and to fill the screen
            let base_zoom_ratio = width / (max * 2.);

            for system in &self.universe {
                let mut point = Point {
                    x: system.x as f32 * base_zoom_ratio + frame.size().width/2. + self.nav.offset.x,
                    y: system.y as f32 * base_zoom_ratio + frame.size().height/2. + self.nav.offset.y,
                };

                point.x = point.x * self.zoom.scale + self.zoom.offset.x;
                point.y = point.y * self.zoom.scale + self.zoom.offset.y;

                if !bounds.contains(point) {
                    continue;
                }

                frame.fill_rectangle(point, Size::new(1., 1.), Color::WHITE);
            }
        });

        vec![geometry]
    }
}
