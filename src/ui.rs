use iced::*;
use space_trader_api::models;

#[derive(Default)]
pub struct App {
    cache: widget::canvas::Cache,
    galaxy: Vec<SizedSystem>,
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

struct SizedWaypoint {
    x: f32,
    y: f32,
    size: f32
}
impl SizedWaypoint {
    pub fn from_system_waypoint(waypoint: models::SystemWaypoint) -> SizedWaypoint {
        SizedWaypoint {
            x: waypoint.x as f32,
            y: waypoint.y as f32,
            size: 3.
        }
    }
}
struct SizedSystem {
    x: f32,
    y: f32,
    size: f32,
    waypoints: Vec<SizedWaypoint>
}
impl SizedSystem {
    fn from_system(system: models::System) -> SizedSystem {
        let waypoints: Vec<SizedWaypoint> = system
            .waypoints
            .into_iter()
            .map(|w| SizedWaypoint::from_system_waypoint(w))
            .collect();

        let size = waypoints.iter().fold(0_f32, |acc, w| {
            let x = w.x.abs();
            let y = w.y.abs();
            acc.max(x.max(y))
        });

        SizedSystem {
            x: system.x as f32,
            y: system.y as f32,
            size,
            waypoints
        }
    }
}

const WINDOW_SIZE: Size = Size::new(1024.0, 768.0);

pub fn run() -> Result {
    App::run(Settings {
        antialiasing: true,
        window: window::Settings {
            size: WINDOW_SIZE,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

impl Application for App {
    type Message = Message;
    type Flags = (i32, i32);
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let galaxy = {
            let text = std::fs::read_to_string("./systems.json").unwrap();
            let systems: Vec<models::System> = serde_json::from_str(&text).unwrap();

            let galaxy_radius = systems
                .iter()
                .fold(0, |state, s| {
                    let x = s.x.abs();
                    let y = s.y.abs();

                    x.max(y).max(state)
                }) as f32;

            let window_radius = WINDOW_SIZE.height.min(WINDOW_SIZE.width);

            // The ratio for the systems to be drawn in the center of the screen and to fill the screen
            let base_scale = window_radius / (galaxy_radius * 2.);

            systems
                .into_iter()
                .map(|s| {
                    let mut s = SizedSystem::from_system(s);
                    s.x = s.x * base_scale + WINDOW_SIZE.width/2.;
                    s.y = s.y * base_scale + WINDOW_SIZE.height/2.;

                    for w in s.waypoints.iter_mut() {
                        w.x *= base_scale;
                        w.y *= base_scale;
                    }

                    s
                })
                .collect()
        };

        (
            App {
                galaxy,
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
                // let scale = (scroll_delta * zoom_coef).max(-1.).min(1.).exp().max(0.01);
                let scale = (scroll_delta * zoom_coef).exp().max(0.01);

                let mouse = self.nav.mouse_current_position - self.nav.offset;

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
                self.nav.offset_start_position = Some(self.nav.mouse_current_position - self.nav.offset);
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
            _ => None
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

            for system in &self.galaxy {
                let point = Point {
                    x: system.x * self.zoom.scale + self.zoom.offset.x + self.nav.offset.x,
                    y: system.y * self.zoom.scale + self.zoom.offset.y + self.nav.offset.y
                };

                if !bounds.contains(point) {
                    continue;
                }

                frame.fill_rectangle(point, Size::new(1., 1.), Color::WHITE);

                if self.zoom.scale > 5. {
                    let waypoints_coords = system
                        .waypoints
                        .iter()
                        .map(|w| Point::new(
                            point.x + (w.x * self.zoom.scale),
                            point.y + (w.y * self.zoom.scale)
                        ));

                    waypoints_coords.for_each(|p| {
                        frame.fill_rectangle(p, Size::new(1., 1.), Color::from_rgb8(255, 0, 0))
                    });
                }
            }
        });

        vec![geometry]
    }
}