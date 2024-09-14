use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::color::Color;
use crate::texture::Texture;

static BALL: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/ball.png")));

#[derive(Debug, Clone)]
pub struct Material {
  pub diffuse: Color,
  pub specular: f32,
  pub albedo: [f32; 4],
  pub refractive_index: f32,
  pub has_texture: bool,
}

impl Material {
  pub fn new(
    diffuse: Color,
    specular: f32,
    albedo: [f32; 4],
    refractive_index: f32,
  ) -> Self {
    Material {
      diffuse,
      specular,
      albedo,
      refractive_index,
      has_texture: false,
    }
  }

  pub fn new_with_texture(
    specular: f32,
    albedo: [f32; 4],
    refractive_index: f32,
  ) -> Self {
    Material {
      diffuse: Color::new(0, 0, 0), // Default color, will be overridden by texture
      specular,
      albedo,
      refractive_index,
      has_texture: true,
    }
  }

  pub fn get_diffuse_color(&mut self, u: f32, v: f32) -> Color {
    if self.has_texture {
      let x = (u * (BALL.width as f32 - 1.0)) as usize;
      let y = ((1.0 - v) * (BALL.height as f32 - 1.0)) as usize;
      BALL.get_color(x, y)
    }
    else {
      self.diffuse
    }
  }

  pub fn black() -> Self {
    Material {
      diffuse: Color::new(0, 0, 0),
      specular: 0.0,
      albedo: [0.0, 0.0, 0.0, 0.0],
      refractive_index: 0.0,
      has_texture: false,
    }
  }
}