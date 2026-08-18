#[derive(Copy, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Purple,
    White
}

impl Color {
    pub fn random() -> Color {
        match rand::random_range(0..7) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Yellow,
            4 => Color::Cyan,
            5 => Color::Purple,
            6 => Color::White,
            _ => Color::White,
        }
    }

    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Red    => (255,   0,   0),
            Color::Green  => (  0, 255,   0),
            Color::Blue   => (  0,   0, 255),
            Color::Yellow => (255, 255,   0),
            Color::Cyan   => (  0, 255, 255),
            Color::Purple => (255,   0, 255),
            Color::White  => (255, 255, 255)
        }
    }

    pub fn to_rgba_array(self) -> [f32; 4] {
        let (r, g, b) = self.to_rgb();

        [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            1.0,
        ]
    }

    // gives us the ability to cycle through colors
    pub fn next(self) -> Color {
        match self {
            Color::Red    => Color::Green,
            Color::Green  => Color::Blue,
            Color::Blue   => Color::Yellow,
            Color::Yellow => Color::Cyan,
            Color::Cyan   => Color::Purple,
            Color::Purple => Color::White,
            Color::White  => Color::Red,
        }
    }
}