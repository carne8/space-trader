use space_trader_api::models::System;

pub fn get_galaxy_radius(galaxy: &Vec<System>) -> f32 {
    galaxy.iter().fold(0_f32, |state, system| {
        let x_square = (system.x as f32).powi(2);
        let y_square = (system.y as f32).powi(2);

        let d = (x_square + y_square).sqrt();
        state.max(d)
    }) as f32
}

pub mod color {
    use space_trader_api::models::SystemType;

    pub fn get_gradient() -> colorgrad::Gradient {
        colorgrad::rd_pu()
    }

    pub fn get_system_color(
        system: &crate::ui::types::SizedSystem,
        galaxy_color: &colorgrad::Gradient,
    ) -> iced::Color {
        match system.r#type {
            SystemType::BlackHole => return iced::Color::BLACK,
            _ => {}
        };

        let raw_color = galaxy_color.at(system.distance as f64);
        iced::Color::from_rgb(
            raw_color.r as f32,
            raw_color.g as f32,
            raw_color.b as f32
        )
    }
}
