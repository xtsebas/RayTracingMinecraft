use std::fmt;
use std::ops::AddAssign;


#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    // Constructor to initialize the color using r, g, b values
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    // Function to create a color from a hex value
    pub const fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Color { r, g, b }
    }

    pub const fn black() -> Self {
        Color { r: 0, g: 0, b: 0 }
    }

    // Function to return the color as a hex value
    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    // Function to return the color as a hex value
    pub fn is_black(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }
}

// Implement addition for Color
use std::ops::Add;

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }
}

// Implement multiplication by a constant for Color
use std::ops::Mul;

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: (self.r as f32 * scalar).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * scalar).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * scalar).clamp(0.0, 255.0) as u8,
        }
    }
}

// Implement display formatting for Color
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
    }
}

impl Color {
    // Método para sumar dos colores
    pub fn add(&self, other: Color) -> Color {
        Color::new(
            (self.r + other.r).min(255),
            (self.g + other.g).min(255),
            (self.b + other.b).min(255),
        )
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, other: Color) {
        self.r = (self.r + other.r).min(255);
        self.g = (self.g + other.g).min(255);
        self.b = (self.b + other.b).min(255);
    }
}

// En tu archivo color.rs o donde esté definida la estructura Color
impl Color {
    pub fn r(&self) -> u8 {
        self.r
    }

    pub fn g(&self) -> u8 {
        self.g
    }

    pub fn b(&self) -> u8 {
        self.b
    }
}
