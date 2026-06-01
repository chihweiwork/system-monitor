// Theme and color management
// Inspired by btop's theming system

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub main_fg: Color,
    pub main_bg: Color,
    pub title: Style,
    pub cpu_box: Style,
    pub mem_box: Style,
    pub cpu_gradient: Vec<Color>,
    pub mem_gradient: Vec<Color>,
    pub graph_color: Color,
    pub text_inactive: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

impl Theme {
    pub fn default_theme() -> Self {
        Self {
            name: "default".to_string(),
            main_fg: Color::White,
            main_bg: Color::Black,
            title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            cpu_box: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            mem_box: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            cpu_gradient: vec![
                Color::Rgb(50, 150, 255),   // Blue for low usage
                Color::Rgb(100, 200, 255),
                Color::Rgb(255, 200, 50),   // Yellow for medium
                Color::Rgb(255, 150, 50),
                Color::Rgb(255, 80, 80),    // Red for high usage
            ],
            mem_gradient: vec![
                Color::Rgb(80, 250, 123),   // Green for low usage
                Color::Rgb(139, 233, 253),  // Cyan for medium
                Color::Rgb(255, 184, 108),  // Orange
                Color::Rgb(255, 121, 198),  // Pink for high
            ],
            graph_color: Color::Cyan,
            text_inactive: Color::DarkGray,
        }
    }

    /// Get color from gradient based on percentage (0.0 to 1.0)
    pub fn gradient_color(gradient: &[Color], percent: f64) -> Color {
        let percent = percent.clamp(0.0, 1.0);
        let idx = (percent * (gradient.len() - 1) as f64) as usize;
        gradient.get(idx).copied().unwrap_or(gradient[0])
    }

    /// Get CPU color based on usage percentage
    pub fn cpu_color(&self, percent: f64) -> Color {
        Self::gradient_color(&self.cpu_gradient, percent / 100.0)
    }

    /// Get memory color based on usage percentage
    pub fn mem_color(&self, percent: f64) -> Color {
        Self::gradient_color(&self.mem_gradient, percent / 100.0)
    }
}
