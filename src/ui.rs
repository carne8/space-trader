mod types;
mod view;
mod galaxy;

use types::*;

use space_trader_api::models;
use iced::{event, executor, mouse, widget, window, Application, Command, Element, Event, Length, Result, Settings, Size, Theme, Vector};


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
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let systems_file = std::fs::read_to_string("./systems.json").unwrap();
        let systems: Vec<models::System> = serde_json::from_str(&systems_file).unwrap();
        let galaxy = {
            let galaxy_radius = galaxy::get_galaxy_radius(&systems);

            // Calculate the base scale for the systems to fit the screen
            let base_scale = {
                let window_size = WINDOW_SIZE.height.min(WINDOW_SIZE.width);
                window_size / (galaxy_radius * 2.)
            };

            let galaxy_color_gradient = galaxy::color::get_gradient();

            systems
                .iter()
                .map(|system| {
                    let mut new_system =
                        SizedSystem::from_system(
                            system,
                            iced::Point::new(WINDOW_SIZE.width/2., WINDOW_SIZE.height/2.),
                            galaxy_radius
                        )
                        .apply_scale(base_scale)      // Scale the system to fit the screen
                        .offset_position(Vector::new( // Center the galaxy
                            WINDOW_SIZE.width/2.,
                            WINDOW_SIZE.height/2.
                        ));

                    // Set the color of the system
                    new_system.color = galaxy::color::get_system_color(
                        &new_system,
                        &galaxy_color_gradient
                    );

                    new_system
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
                let zoom_sensibility = 0.1;
                let scale = (scroll_delta * zoom_sensibility).exp().max(0.01);

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