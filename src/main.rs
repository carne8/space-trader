mod space_traders;

use std::{default, fs};

use serde::Deserialize;

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

use iced::{
    event, executor, mouse::*, widget::{canvas::{self, Cache}, Canvas}, Application, Color, Command, Element, Length, Point, Renderer, Settings, Size, Subscription, Theme, Transformation, Vector
};

pub fn main() -> iced::Result {
    Example::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct Example {
    cache: Cache,
    universe: Vec<space_traders::types::System>,
    offset: Vector,
    mouse_current_position: Point,
    offset_start_position: Option<Point>,
    zoom: f32,
    zoom_position: Option<Point>,
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
            zoom: 0.,
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
                self.zoom = (self.zoom + scroll_delta * zoom_coef).max(-10.);
                self.zoom_position = Some(self.mouse_current_position);
            },
            Message::MouseDown => {
                self.offset_start_position = Some(
                    Point {
                        x: self.offset.x + self.mouse_current_position.x,
                        y: self.offset.y + self.mouse_current_position.y
                    }
                );
            }
            Message::MouseUp => {
                self.offset_start_position = None;
            }
            Message::MouseMoved(position) => {
                self.mouse_current_position = position;

                if let Some(mouse_down_position) = self.offset_start_position {
                    self.offset = mouse_down_position - position;
                }
            }
        }

        self.cache.clear();
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        event::listen_with(|event, _| match event {
            iced::Event::Mouse(mouse_event) => match mouse_event {
                Event::WheelScrolled { delta: ScrollDelta::Pixels { x: _, y } }
                | Event::WheelScrolled { delta: ScrollDelta::Lines { x: _, y } } => Some(Message::MouseWheelScrolled(y)),

                Event::ButtonPressed(Button::Left) => Some(Message::MouseDown),
                Event::ButtonReleased(Button::Left) => Some(Message::MouseUp),

                Event::CursorMoved { position } => Some(Message::MouseMoved(position)),
                _ => None,
            },
            _ => None,
        })
    }
}

impl canvas::Program<Message> for Example {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<<Renderer as canvas::Renderer>::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let zoom_position = self.zoom_position.unwrap_or(center);

            let width = frame.width().min(frame.height());
            let max = self.universe.iter().map(|system| system.x.max(system.y)).max().unwrap() as f32;

            // The ratio for the systems to be drawn in the center of the screen and to fill the screen
            let base_zoom_ratio = width / max;
            let zoom = (self.zoom.exp() - 1.).max(-0.999);

            println!("Zoom: {}", zoom);

            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);

            for system in &self.universe {
                let x = system.x as f32;
                let y = system.y as f32;

                let apply_zoom = |value: f32, pos: f32, zoom: f32| (value - pos) * zoom + value;

                // let point = Point {
                //     x: center.x - self.offset.x + x * base_zoom_ratio * zoom,
                //     y: center.y - self.offset.y + y * base_zoom_ratio * zoom
                // };

                let normalized_pos = Point {
                    x: x,
                    y: y
                };

                let point = Point {
                    x: apply_zoom(normalized_pos.x - self.offset.x, zoom_position.x, zoom) * base_zoom_ratio + center.x,
                    y: apply_zoom(normalized_pos.y - self.offset.y, zoom_position.y, zoom) * base_zoom_ratio + center.y
                };

                if !bounds.contains(point) {
                    continue;
                }

                frame.fill_rectangle(point, Size::new(1., 1.), Color::WHITE);
            }
        });

        vec![geometry]
    }
}