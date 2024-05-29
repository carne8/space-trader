use iced::*;
use iced::widget::canvas::{Path, Fill};
use space_trader_api::models;
use widget::canvas::Style;

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
            size: 30.
        }
    }

    pub fn apply_scale(&self, scale: f32) -> SizedWaypoint {
        SizedWaypoint {
            x: self.x * scale,
            y: self.y * scale,
            size: self.size * scale
        }
    }
}

const GALAXY_COLOR_0: Color = color!(228, 148, 152);
const GALAXY_COLOR_1: Color = color!(166, 194, 221);
const GALAXY_COLOR_2: Color = color!(39, 47, 65);

struct SizedSystem {
    x: f32,
    y: f32,
    size: f32,
    r#type: models::SystemType,
    waypoints: Vec<SizedWaypoint>,
    color: Color
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
            r#type: system.r#type,
            waypoints,
            color: Color::WHITE
        }
    }
    pub fn apply_scale(&self, scale: f32) -> SizedSystem {
        SizedSystem {
            x: self.x * scale,
            y: self.y * scale,
            size: self.size * scale,
            r#type: self.r#type,
            waypoints: self.
                waypoints
                .iter()
                .map(|w| w.apply_scale(scale))
                .collect(),
            color: self.color
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
                    let mut new_s = SizedSystem::from_system(s.clone()).apply_scale(base_scale);
                    new_s.x += WINDOW_SIZE.width/2.;
                    new_s.y += WINDOW_SIZE.height/2.;

                    let color = {
                        let system_relative_pos = {
                            let x = s.x as f32;
                            let y = s.y as f32;
                            let distance_from_center = (x.powi(2) + y.powi(2)).sqrt();
                            distance_from_center / galaxy_radius
                        };
                        let r =
                            (1.-system_relative_pos) * GALAXY_COLOR_1.r
                            + system_relative_pos * GALAXY_COLOR_0.r;

                        let g =
                            (1.-system_relative_pos) * GALAXY_COLOR_1.g
                            + system_relative_pos * GALAXY_COLOR_0.g;

                        let b =
                            (1.-system_relative_pos) * GALAXY_COLOR_1.b
                            + system_relative_pos * GALAXY_COLOR_0.b;


                        color!(r * 255., g * 255., b * 255.)
                    };

                    new_s.color = color;
                    new_s
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
                let zoomed_system = system.apply_scale(self.zoom.scale);
                let point = Point {
                    x: zoomed_system.x + self.zoom.offset.x + self.nav.offset.x,
                    y: zoomed_system.y + self.zoom.offset.y + self.nav.offset.y
                };

                if !bounds.expand(zoomed_system.size).contains(point) {
                    continue;
                }

                if self.zoom.scale > 8. {
                    frame.fill(
                        &Path::circle(point, zoomed_system.size),
                        Fill {
                            style: Style::Solid(zoomed_system.color),
                            ..Fill::default()
                        }
                    );

                    for w in zoomed_system.waypoints {
                        frame.fill_rectangle(
                            Point::new(
                                point.x + w.x - w.size/2.,
                                point.y + w.y - w.size/2.
                            ),
                            Size::new(w.size, w.size),
                            Color::WHITE
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