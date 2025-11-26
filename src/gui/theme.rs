use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.94, 0.94, 0.96),
            Theme::Dark => Color::from_rgb(0.08, 0.08, 0.10), // Very dark background
        }
    }

    pub fn surface(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.97, 0.97, 0.98),
            Theme::Dark => Color::from_rgb(0.18, 0.18, 0.20), // Slightly lighter dark surface
        }
    }
    
    pub fn card_background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(1.0, 1.0, 1.0),
            Theme::Dark => Color::from_rgb(0.22, 0.22, 0.24), // Elevated card background - subtle difference
        }
    }
    
    #[allow(dead_code)]
    pub fn panel_background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.95, 0.95, 0.97),
            Theme::Dark => Color::from_rgb(0.16, 0.16, 0.18), // Sidebar/panel background
        }
    }

    pub fn text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.05, 0.05, 0.05),
            Theme::Dark => Color::from_rgb(1.0, 1.0, 1.0), // White text for dark theme (non-yellow backgrounds)
        }
    }

    pub fn secondary_text(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.35, 0.35, 0.4),
            Theme::Dark => Color::from_rgb(0.9, 0.9, 0.9), // Light gray text for dark theme
        }
    }

    pub fn primary(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.7, 0.6, 0.2), // Calm yellow for light theme
            Theme::Dark => Color::from_rgb(0.95, 0.9, 0.45), // Very vibrant yellow-green like classic media players
        }
    }

    pub fn danger(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.9, 0.2, 0.2),
            Theme::Dark => Color::from_rgb(1.0, 0.3, 0.3),
        }
    }
}



