use space_trader_api::models;
use iced::{widget, Color, Point, Vector};

#[derive(Default)]
pub struct App {
    pub cache: widget::canvas::Cache,
    pub galaxy: Vec<SizedSystem>,
    pub zoom: Zoom,
    pub nav: Navigation
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    MouseDown,
    MouseUp,
    MouseMoved(Point),
    MouseWheelScrolled(f32),
}

/// The zoom state
pub struct Zoom {
    pub scale: f32,
    pub offset: Vector
}
impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            scale: 1.,
            offset: Vector::ZERO
        }
    }
}

/// The navigation state (the offset of the galaxy)
#[derive(Default)]
pub struct Navigation {
    pub offset_start_position: Option<Point>,
    pub mouse_current_position: Point,
    pub offset: Vector
}

/// A waypoint with coordinates and size
pub struct SizedWaypoint {
    pub x: f32,
    pub y: f32,
    pub size: f32
}
impl SizedWaypoint {
    pub fn from_system_waypoint(waypoint: &models::SystemWaypoint) -> SizedWaypoint {
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

/// A system with coordinates, size, type, waypoints and color
/// It's based on the `space_trader_api::models::System` struct
pub struct SizedSystem {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    /// The distance from the center of the galaxy to the system from 0 to 1
    pub distance: f32,
    pub r#type: models::SystemType,
    pub waypoints: Vec<SizedWaypoint>,
    pub color: Color
}
impl SizedSystem {
    pub fn from_system(system: &models::System, galaxy_center: Point, galaxy_radius: f32) -> SizedSystem {
        // Convert the waypoints to SizedWaypoint
        let waypoints: Vec<SizedWaypoint> = system
            .waypoints
            .iter()
            .map(|w| SizedWaypoint::from_system_waypoint(w))
            .collect();

        // Calculate the size of the system based on the waypoints
        // Take the the max orbit radius of the waypoints
        let size = waypoints.iter().fold(0_f32, |acc, w| {
            let x = w.x.abs();
            let y = w.y.abs();
            acc.max(x.max(y))
        });

        // Calculate the distance from the center of the galaxy to the system
        let x_square = (system.x as f32 - galaxy_center.x).powi(2);
        let y_square = (system.y as f32 - galaxy_center.y).powi(2);

        let distance = (x_square + y_square).sqrt() / galaxy_radius;

        SizedSystem {
            x: system.x as f32,
            y: system.y as f32,
            size,
            distance,
            r#type: system.r#type,
            waypoints,
            color: Color::WHITE
        }
    }

    /// Apply a scale to the system and its waypoints
    pub fn apply_scale(&self, scale: f32) -> SizedSystem {
        SizedSystem {
            x: self.x * scale,
            y: self.y * scale,
            size: self.size * scale,
            distance: self.distance,
            r#type: self.r#type,
            waypoints: self.
                waypoints
                .iter()
                .map(|w| w.apply_scale(scale))
                .collect(),
            color: self.color
        }
    }

    /// Offset the position of the system
    pub fn offset_position(mut self, offset: Vector) -> SizedSystem {
        self.x += offset.x;
        self.y += offset.y;
        // Don't change the waypoint positions
        // because they are relative to the system position

        self
    }
}
