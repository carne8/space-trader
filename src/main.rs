#![allow(dead_code)]

mod space_traders;

use std::{fs, result::Result};
use iced::keyboard;
use iced::*;

use serde::Deserialize;
use space_traders::types::Symbol;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetAgentResponse {
    data: space_traders::types::Agent
}

const TOKEN: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiQ0FSTkU4IiwidmVyc2lvbiI6InYyLjIuMCIsInJlc2V0X2RhdGUiOiIyMDI0LTA1LTE5IiwiaWF0IjoxNzE2NTYwODYzLCJzdWIiOiJhZ2VudC10b2tlbiJ9.aDsB9OhPmg9Q6cN8MgyAOL5PRKVAuFzbVmNPwOjvrJ78OUkRA0oACTqXoVYm7yql1D_rDDhDSJvqb--qg5qcY73zYhE0-0qnJzO3UHaBCj9bhuSTu0-XkaydT8exgV_BlHA1tLo3mh9eg_16fawJuba7gq-PY8FE95P0SSOyJ67HBPh9DfbxyJu5E6FajBoCCe_cA954jpAM70zNa15mcIKbYw-6bLvIFPTvzDm6tHD3FaneOxTCoxv-Y8hP9e_bIuPVGBQLvv6wSg9mZN61kQSY_vtjM73GiPNpPG0te86UWhbvdBC6qpZfEMnxngXqpsrC0pLqtlZlfUVCAsImvw";

async fn download_systems() -> Result<(), String> {
    let agent = space_traders::get_agent(TOKEN)
        .await
        .map_err(|err| format!("Failed to get agent: {err}"))?;

    println!("{:?}", agent);

    let systems = space_traders::get_systems(TOKEN)
        .await
        .map_err(|err| format!("Failed to get systems: {err}"))?;

    let systems_json = serde_json::to_string(&systems).unwrap();
    std::fs::write("./systems.json", systems_json).unwrap();

    Ok(())
}

// #[tokio::main]
// async fn main() -> Result<(), String> {
//     let app = iced::application::Application::new();

//     Ok(())
// }

pub fn main() -> iced::Result {
    Example::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
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

#[derive(Default)]
struct Example {
    cache: widget::canvas::Cache,
    universe: Vec<space_traders::types::System>,
    zoom: Zoom,
    nav: Navigation
}

#[derive(Debug, Clone, Copy)]
enum Message {
    MouseDown,
    MouseUp,
    MouseMoved(Point),
    MouseWheelScrolled(f32),
}

impl Application for Example {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let universe = {
            let text = fs::read_to_string("./systems.json").unwrap();
            let systems = serde_json::from_str(&text).unwrap();

            systems
        };

        (Example {
            universe,
            ..Example::default()
        }, Command::none())
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
                        self.zoom.offset.y * scale + (mouse.y - mouse.y * scale)
                    )
                };
            },
            Message::MouseDown => {
                self.nav.offset_start_position = Some(
                    Point {
                        x: self.nav.mouse_current_position.x - self.nav.offset.x,
                        y: self.nav.mouse_current_position.y - self.nav.offset.y
                    }
                );
            }
            Message::MouseUp => {
                self.nav.offset_start_position = None;
            }
            Message::MouseMoved(position) => {
                self.nav.mouse_current_position = position;

                if let Some(mouse_down_position) = self.nav.offset_start_position {
                    self.nav.offset = position - mouse_down_position;
                }
            }
        }

        self.cache.clear();
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
            Event::Keyboard(keyboard::Event::KeyPressed { key, location: _, modifiers: _, text: _ }) => {
                match key.as_ref() {
                    keyboard::Key::Character("=") => Some(Message::MouseWheelScrolled(1.)),
                    keyboard::Key::Character("-") => Some(Message::MouseWheelScrolled(-1.)),
                    _ => None
                }
            },

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

impl widget::canvas::Program<Message> for Example {
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
            let max = self.universe.iter().map(|system| system.x.max(system.y)).max().unwrap() as f32;

            // The ratio for the systems to be drawn in the center of the screen and to fill the screen
            let base_zoom_ratio = width / max;

            for system in &self.universe {
                let mut point = Point {
                    x: system.x as f32 * base_zoom_ratio + self.nav.offset.x,
                    y: system.y as f32 * base_zoom_ratio + self.nav.offset.y
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