use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::time::Instant;
// use rand::{Rng};

mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod texture;
mod cube;

use framebuffer::Framebuffer;
use color::Color;
use ray_intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use material::Material;
use cube::Cube;
use texture::Texture;

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);
const NIGHT_COLOR: Color = Color::new(10, 10, 50);

static RUBBER_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wood.jpg")));
static OLDWOOD_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/oldwood.png")));
static DOOR_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/door.png")));
static GLASS_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/glass.png")));
static STONE_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/cobblestone.jpg")));
static SAND_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/sand.jpg")));

//Materiales
pub static RUBBER: Lazy<Material> = Lazy::new(|| {
    Material::new_with_texture(
        50.0,                        // Specular
        [0.9, 0.1, 0.0, 0.0],        // Albedo
        0.0,                         // Refractive index
        RUBBER_TEXTURE.clone(),        // Textura para el material
        None,
        0.0
    )
});
pub static OLDWOOD: Lazy<Material> = Lazy::new(|| {
    Material::new_with_texture(
        50.0,                        // Specular
        [0.9, 0.1, 0.0, 0.0],        // Albedo
        0.0,                         // Refractive index
        OLDWOOD_TEXTURE.clone(),        // Textura para el material
        None,
        0.0
    )
});
pub static DOOR : Lazy<Material> = Lazy::new(|| {
    Material::new_with_texture(
        50.0,                        // Specular
        [0.9, 0.1, 0.0, 0.0],        // Albedo
        0.0,                         // Refractive index
        DOOR_TEXTURE.clone(),        // Textura para el material
        Some(Color::new(255, 255, 255)),  // Color de emisión
        1.0
    )
});
pub static GLASS : Lazy<Material> = Lazy::new(|| {
    Material::new_with_texture(
        50.0,
        [0.9, 0.1, 0.0, 0.5],        // Albedo (último valor es el componente alfa para transparencia)
        1.5,                    // Refractive index
        GLASS_TEXTURE.clone(),        // Textura para el material
        None,
        0.0
    )
});
pub static STONE : Lazy<Material> = Lazy::new(|| {
    Material::new_with_texture(
        50.0,                        // Specular
        [0.9, 0.1, 0.0, 0.0],        // Albedo
        0.0,                   // Refractive index
        STONE_TEXTURE.clone(),        // Textura para el material
        None,
        0.0
    )
});
pub static SAND : Lazy<Material> = Lazy::new(|| {
    Material::new_with_texture(
        50.0,                        // Specular
        [0.9, 0.1, 0.0, 0.0],        // Albedo
        0.0,                   // Refractive index
        SAND_TEXTURE.clone(),        // Textura para el material
        None,
        0.0
    )
});

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        // Ray is entering the object
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        // Ray is leaving the object
        n_cosi = cosi;
        eta = eta_t;  // Assuming it's going back into air with index 1.0
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        // Total internal reflection
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(
    intersect: &Intersect,
    objects: &[Cube],
    light_dir: &Vec3,
    light_distance: f32
) -> f32 {
    let shadow_ray_origin = offset_origin(intersect, light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            // Si el objeto intersectado emite luz, reduce la sombra, pero no la elimina completamente
            if let Some(_emission) = object.material.emission_color {
                let distance_ratio = shadow_intersect.distance / light_distance;
                let emission_intensity = 1.0 / (distance_ratio * distance_ratio);
                shadow_intensity = emission_intensity; // Ajustar la sombra según la intensidad de la emisión
                break; // Asumimos que el bloque emisor de luz bloquea cualquier otra sombra
            } else {
                // Si no es un emisor de luz, aplica la sombra normalmente
                shadow_intensity = 1.0;
                break;
            }
        }
    }

    shadow_intensity
}

fn generate_random_direction() -> Vec3 {
    let theta = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let z: f32 = rand::random::<f32>() * 2.0 - 1.0;  // Random valor entre -1 y 1
    let r = (1.0 - z * z).sqrt();
    let x = r * theta.cos();
    let y = r * theta.sin();
    Vec3::new(x, y, z).normalize()
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    light: &Light,
    depth: u32,
    SKY_COLOR: Color
) -> Color {
    if depth > 3 {
        return SKY_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    // Buscar la intersección más cercana
    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    // Si no hay intersecciones, devuelve el color del cielo
    if !intersect.is_intersecting {
        return SKY_COLOR;
    }

    // Cálculo de la dirección de la luz principal
    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    // Calcular la intensidad de la sombra para la luz principal
    let light_distance = (light.position - intersect.point).magnitude();
    let shadow_intensity = cast_shadow(&intersect, objects, &light_dir, light_distance);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    // Cálculo de iluminación difusa y especular para la luz principal
    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse_color = intersect.material.get_diffuse_color(intersect.u, intersect.v);
    let diffuse = diffuse_color * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    // Manejo de reflexión
    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1, SKY_COLOR);
    }

    // Manejo de refracción
    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1, SKY_COLOR);
    }

    // **Propagación de la luz emitida omnidireccionalmente**
    let mut emission_contribution = Color::black();
    for object in objects {
        if let Some(emission) = object.material.emission_color {
            let num_rays = 16;  // Número de direcciones para emitir luz
            let emission_strength = 1.0 / (num_rays as f32);  // Reducir la intensidad de emisión

            for _ in 0..num_rays {
                // Generar una dirección aleatoria usando la función `generate_random_direction`
                let emission_dir = generate_random_direction();

                // Calcular la distancia de emisión
                let emission_origin = object.position();  // Punto de emisión en el centro del cubo
                let emission_distance = (emission_origin - intersect.point).magnitude();

                // Verificar si este objeto está bloqueando la luz
                let shadow_intensity = cast_shadow(&intersect, objects, &emission_dir, emission_distance);
                let emission_intensity = emission_strength / (emission_distance * emission_distance) * (1.0 - shadow_intensity);

                // Aplicar iluminación difusa desde el objeto emisor
                let emission_diffuse_intensity = intersect.normal.dot(&emission_dir).max(0.0);
                let emission_diffuse = emission * emission_diffuse_intensity * emission_intensity;

                // Acumular la contribución de esta fuente de luz en esa dirección
                emission_contribution = emission_contribution + emission_diffuse;
            }
        }
    }

    // Retorna el color final, sumando las contribuciones de la luz emitida
    (diffuse + specular + emission_contribution) * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light, SKY_COLOR: Color) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    // random number generator
    // let mut rng = rand::thread_rng();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // if rng.gen_range(0.0..1.0) < 0.9 {
            //      continue;
            // }

            // Map the pixel coordinate to screen space [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio and perspective 
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Apply camera rotation to the ray direction
            let rotated_direction = camera.basis_change(&ray_direction);

            // Cast the ray and get the pixel color
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0, SKY_COLOR);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn handle_sky_color(keys: &Vec<Key>, sky_color: &mut Color, light: &mut Light) {
    for key in keys {
        match key {
            Key::N => {
                *sky_color = NIGHT_COLOR;       // Cambia a color de noche
                light.intensity = 0.3;          // Luz más tenue de noche
                light.color = Color::new(150, 150, 200);  // Color de luz más frío
            },
            Key::D => {
                *sky_color = SKYBOX_COLOR;         // Cambia a color de día
                light.intensity = 1.0;          // Luz más brillante durante el día
                light.color = Color::new(255, 255, 255);  // Color de luz más cálido
            },
            _ => {}
        }
    }
}


fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Diorama",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    let mut objects = vec![
        // Columna de 4 bloques
        Cube { min: Vec3::new(-8.0, 7.0, -1.0), max: Vec3::new(-6.0, 5.0, 1.0), material: (*STONE).clone() }, // Bloque superior
        Cube { min: Vec3::new(-8.0, 5.0, -1.0), max: Vec3::new(-6.0, 3.0, 1.0), material: (*OLDWOOD).clone() }, // Segundo bloque
        Cube { min: Vec3::new(-8.0, 3.0, -1.0), max: Vec3::new(-6.0, 1.0, 1.0), material: (*OLDWOOD).clone() }, // Tercer bloque
        Cube { min: Vec3::new(-8.0, 1.0, -1.0), max: Vec3::new(-6.0, -1.0, 1.0), material: (*OLDWOOD).clone() }, // Bloque inferior

        Cube { min: Vec3::new(-8.0, 7.0, -9.0), max: Vec3::new(-6.0, 5.0, -7.0), material: (*STONE).clone() }, // Bloque superior
        Cube { min: Vec3::new(-8.0, 5.0, -9.0), max: Vec3::new(-6.0, 3.0, -7.0), material: (*OLDWOOD).clone() },
        Cube { min: Vec3::new(-8.0, 3.0, -9.0), max: Vec3::new(-6.0, 1.0, -7.0), material: (*OLDWOOD).clone() }, // Tercer bloque
        Cube { min: Vec3::new(-8.0, 1.0, -9.0), max: Vec3::new(-6.0, -1.0, -7.0), material: (*OLDWOOD).clone() }, // Bloque inferior

        //pared izquierda
        Cube { min: Vec3::new(-2.0, 3.0, -1.0), max: Vec3::new(0.0, 5.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-4.0, 3.0, -1.0), max: Vec3::new(-2.0, 5.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-6.0, 3.0, -1.0), max: Vec3::new(-4.0, 5.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-6.0, 1.0, -1.0), max: Vec3::new(-4.0, 3.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-6.0, -1.0, -1.0), max: Vec3::new(-4.0, 1.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-4.0, 1.0, -1.0), max: Vec3::new(-2.0, 3.0, 1.0), material: (*GLASS).clone() },
        Cube { min: Vec3::new(-4.0, -1.0, -1.0), max: Vec3::new(-2.0, 1.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-2.0, -1.0, -1.0), max: Vec3::new(0.0, 1.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-2.0, 1.0, -1.0), max: Vec3::new(0.0, 3.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(0.0, 3.0, -1.0), max: Vec3::new(2.0, 5.0, 1.0), material: (*RUBBER).clone() },
        
        Cube { min: Vec3::new(-2.0, 3.0, -9.0), max: Vec3::new(0.0, 5.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-4.0, 3.0, -9.0), max: Vec3::new(-2.0, 5.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-6.0, 3.0, -9.0), max: Vec3::new(-4.0, 5.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-6.0, 1.0, -9.0), max: Vec3::new(-4.0, 3.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-6.0, -1.0, -9.0), max: Vec3::new(-4.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-4.0, 1.0, -9.0), max: Vec3::new(-2.0, 3.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-4.0, -1.0, -9.0), max: Vec3::new(-2.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-2.0, -1.0, -9.0), max: Vec3::new(0.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-2.0, 1.0, -9.0), max: Vec3::new(0.0, 3.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(0.0, 3.0, -9.0), max: Vec3::new(2.0, 5.0, -7.0), material: (*RUBBER).clone() },

        //puerta
        Cube { min: Vec3::new(0.0, -1.0, -1.0), max: Vec3::new(2.0, 3.0, 1.0), material: (*DOOR).clone() },

        Cube { min: Vec3::new(0.0, -1.0, -9.0), max: Vec3::new(2.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(0.0, 1.0, -9.0), max: Vec3::new(2.0, 3.0, -7.0), material: (*RUBBER).clone() },
        
        //pared derecha
        Cube { min: Vec3::new(6.0, 3.0, -1.0), max: Vec3::new(8.0, 5.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(4.0, 3.0, -1.0), max: Vec3::new(6.0, 5.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(2.0, 3.0, -1.0), max: Vec3::new(4.0, 5.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(2.0, 1.0, -1.0), max: Vec3::new(4.0, 3.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(2.0, -1.0, -1.0), max: Vec3::new(4.0, 1.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(4.0, 1.0, -1.0), max: Vec3::new(6.0, 3.0, 1.0), material: (*GLASS).clone() },
        Cube { min: Vec3::new(4.0, -1.0, -1.0), max: Vec3::new(6.0, 1.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(6.0, -1.0, -1.0), max: Vec3::new(8.0, 1.0, 1.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(6.0, 1.0, -1.0), max: Vec3::new(8.0, 3.0, 1.0), material: (*RUBBER).clone() },

        Cube { min: Vec3::new(6.0, 3.0, -9.0), max: Vec3::new(8.0, 5.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(4.0, 3.0, -9.0), max: Vec3::new(6.0, 5.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(2.0, 3.0, -9.0), max: Vec3::new(4.0, 5.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(2.0, 1.0, -9.0), max: Vec3::new(4.0, 3.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(2.0, -1.0, -9.0), max: Vec3::new(4.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(4.0, 1.0, -9.0), max: Vec3::new(6.0, 3.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(4.0, -1.0, -9.0), max: Vec3::new(6.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(6.0, -1.0, -9.0), max: Vec3::new(8.0, 1.0, -7.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(6.0, 1.0, -9.0), max: Vec3::new(8.0, 3.0, -7.0), material: (*RUBBER).clone() },

        // Columna de 4 bloques (a la derecha)
        Cube { min: Vec3::new(8.0, 5.0, -1.0), max: Vec3::new(10.0, 7.0, 1.0), material: (*STONE).clone() }, // Bloque superior
        Cube { min: Vec3::new(8.0, 3.0, -1.0), max: Vec3::new(10.0, 5.0, 1.0), material: (*OLDWOOD).clone() }, // Segundo bloque
        Cube { min: Vec3::new(8.0, 1.0, -1.0), max: Vec3::new(10.0, 3.0, 1.0), material: (*OLDWOOD).clone() }, // Tercer bloque
        Cube { min: Vec3::new(8.0, -1.0, -1.0), max: Vec3::new(10.0, 1.0, 1.0), material: (*OLDWOOD).clone() }, // Bloque inferior

        Cube { min: Vec3::new(8.0, 5.0, -9.0), max: Vec3::new(10.0, 7.0, -7.0), material: (*STONE).clone() }, // Bloque superior
        Cube { min: Vec3::new(8.0, 3.0, -9.0), max: Vec3::new(10.0, 5.0, -7.0), material: (*OLDWOOD).clone() }, // Segundo bloque
        Cube { min: Vec3::new(8.0, 1.0, -9.0), max: Vec3::new(10.0, 3.0, -7.0), material: (*OLDWOOD).clone() }, // Tercer bloque
        Cube { min: Vec3::new(8.0, -1.0, -9.0), max: Vec3::new(10.0, 1.0, -7.0), material: (*OLDWOOD).clone() }, // Bloque inferior

        // pared lateral izquierda
        Cube { min: Vec3::new(-8.0, 5.0, -5.0), max: Vec3::new(-6.0, 3.0, -3.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(-8.0, 5.0, -3.0), max: Vec3::new(-6.0, 3.0, -1.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(-8.0, 3.0, -5.0), max: Vec3::new(-6.0, 1.0, -3.0), material: (*GLASS).clone() }, 
        Cube { min: Vec3::new(-8.0, 3.0, -3.0), max: Vec3::new(-6.0, 1.0, -1.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(-8.0, 1.0, -3.0), max: Vec3::new(-6.0, -1.0, -1.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(-8.0, 1.0, -5.0), max: Vec3::new(-6.0, -1.0, -3.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(-8.0, 1.0, -7.0), max: Vec3::new(-6.0, -1.0, -5.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(-8.0, 3.0, -7.0), max: Vec3::new(-6.0, 1.0, -5.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(-8.0, 5.0, -7.0), max: Vec3::new(-6.0, 3.0, -5.0), material: (*RUBBER).clone() }, 

        // pared lateral Derecha
        Cube { min: Vec3::new(8.0, 5.0, -5.0), max: Vec3::new(10.0, 3.0, -3.0), material: (*RUBBER).clone() },
        Cube { min: Vec3::new(8.0, 5.0, -3.0), max: Vec3::new(10.0, 3.0, -1.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(8.0, 3.0, -5.0), max: Vec3::new(10.0, 1.0, -3.0), material: (*GLASS).clone() }, 
        Cube { min: Vec3::new(8.0, 3.0, -3.0), max: Vec3::new(10.0, 1.0, -1.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(8.0, 1.0, -3.0), max: Vec3::new(10.0, -1.0, -1.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(8.0, 1.0, -5.0), max: Vec3::new(10.0, -1.0, -3.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(8.0, 1.0, -7.0), max: Vec3::new(10.0, -1.0, -5.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(8.0, 3.0, -7.0), max: Vec3::new(10.0, 1.0, -5.0), material: (*RUBBER).clone() }, 
        Cube { min: Vec3::new(8.0, 5.0, -7.0), max: Vec3::new(10.0, 3.0, -5.0), material: (*RUBBER).clone() }, 
    ];

    // Techo
    for z in (-9.0 as i32..=-1.0 as i32).step_by(2.0 as usize) {
        for x in (-8.0 as i32..=8.0 as i32).step_by(2.0 as usize) {
            // Genera los cubos en las posiciones del perímetro
            objects.push(Cube {
                min: Vec3::new(x as f32, 5.0, z as f32),
                max: Vec3::new((x + 2) as f32, 7.0, (z + 2) as f32),
                material: (*STONE).clone(),
            });
        }
    }

    // Piso
    for z in (-13.0 as i32..=5.0 as i32).step_by(2.0 as usize) {
        for x in (-14.0 as i32..=14.0 as i32).step_by(2.0 as usize) {
            // Genera los cubos en las posiciones del perímetro
            objects.push(Cube {
                min: Vec3::new(x as f32, -3.0, z as f32),
                max: Vec3::new((x + 2) as f32, -1.0, (z + 2) as f32),
                material: (*SAND).clone(),
            });
        }
    }

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(10.0, 10.0, 30.0),  // eye: Arriba a la derecha y alejada
        Vec3::new(0.0, 0.0, 0.0),     // center: Apuntando al origen
        Vec3::new(0.0, 1.0, 0.0),     // up: Vector "up" del mundo
        Vec3::new(0.0, 0.0, 0.0),     // pos: Inicialmente en el origen
        Vec3::new(0.0, 0.0, -1.0),    // dir: Mirando hacia adelante (en -Z)
        60.0,                         // fov: Campo de visión de la cámara
    );

    let rotation_speed = PI/50.0;

    let mut light = Light::new(
        Vec3::new(20.0, 20.0, 20.0),
        Color::new(255, 255, 255),
        1.0
    );

    let mut SKY_COLOR = SKYBOX_COLOR;

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::W) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::S) {
            camera.orbit(0.0, rotation_speed);
        }

        if window.is_key_down(Key::Up) {
            camera.zoom(0.5);  // Acercar cámara (zoom in)
        }
    
        if window.is_key_down(Key::Down) {
            camera.zoom(-0.5); // Alejar cámara (zoom out)
        }

        // handle sky color change
        if window.is_key_down(Key::N) || window.is_key_down(Key::D) {
            handle_sky_color(&window.get_keys(), &mut SKY_COLOR, &mut light);
            render(&mut framebuffer, &objects, &camera, &mut light, SKY_COLOR);
        }

        if camera.is_changed() {
            // Render the scene
            render(&mut framebuffer, &objects, &camera, &mut light, SKY_COLOR);
        }

        // update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}