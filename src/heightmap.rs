use bevy::{
    math::Vec3,
    prelude::{Color, Image, Mesh},
    render::{
        mesh::Indices,
        render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    },
    utils::{Entry, HashMap},
};
use bracket_noise::prelude::*;
use tracing::instrument;

const WATER_LEVEL: f32 = -10.0;
const PLATEAU_LEVEL: f32 = 0.25;
pub const LOW_DEF: u32 = 5;
pub const HIGH_DEF: u32 = 50;
const FLATTENING: f32 = 5.0;

pub struct HeightMap {
    seeds: crate::terra::TerraNoises,
    x: f32,
    y: f32,
}

struct CachedNoise {
    cache: HashMap<(i32, i32), f32>,
    noise: FastNoise,
}

impl CachedNoise {
    fn new(noise: FastNoise) -> Self {
        Self {
            cache: HashMap::default(),
            noise,
        }
    }
    fn get_noise(&mut self, x: f32, y: f32, i: i32, j: i32) -> f32 {
        match self.cache.entry((i, j)) {
            Entry::Occupied(value) => *value.get(),
            Entry::Vacant(vacant) => {
                let noise = self.noise.get_noise(x, y);
                vacant.insert(noise);
                noise
            }
        }
    }
}

impl HeightMap {
    #[instrument(skip(seeds))]
    pub fn build_heightmap(x: f32, y: f32, seeds: crate::terra::TerraNoises) -> Self {
        Self { seeds, x, y }
    }

    fn is_obstacle(elevation: f32) -> bool {
        !(WATER_LEVEL..=PLATEAU_LEVEL).contains(&elevation)
    }

    fn pretty_border(kind: u8, elevation_block: f32, x: f32, y: f32, elevation: f32) -> f32 {
        match kind {
            0 => elevation / FLATTENING as f32,
            3 => {
                if x < -y {
                    Self::obstacle_height(elevation_block, elevation)
                } else {
                    elevation / FLATTENING as f32
                }
            }
            6 => {
                if x < y {
                    Self::obstacle_height(elevation_block, elevation)
                } else {
                    elevation / FLATTENING as f32
                }
            }
            9 => {
                if x > y {
                    Self::obstacle_height(elevation_block, elevation)
                } else {
                    elevation / FLATTENING as f32
                }
            }
            12 => {
                if -x < y {
                    Self::obstacle_height(elevation_block, elevation)
                } else {
                    elevation / FLATTENING as f32
                }
            }
            _ => Self::obstacle_height(elevation_block, elevation),
        }
    }

    fn obstacle_height(elevation_block: f32, elevation: f32) -> f32 {
        if elevation_block > PLATEAU_LEVEL as f32 {
            (elevation - (1.0 - PLATEAU_LEVEL as f32)) / FLATTENING as f32 + 0.4
        } else {
            elevation
        }
    }

    #[instrument(skip(self))]
    pub fn into_mesh_and_texture(self) -> Terrain {
        let mut elevation_noise = FastNoise::seeded(self.seeds.elevation_seed as u64);
        elevation_noise.set_noise_type(NoiseType::PerlinFractal);
        elevation_noise.set_fractal_type(FractalType::FBM);
        elevation_noise.set_fractal_octaves(7);
        elevation_noise.set_fractal_gain(0.4);
        elevation_noise.set_fractal_lacunarity(2.0);
        elevation_noise.set_frequency(2.0);
        let mut simplified_elevation_noise = FastNoise::seeded(self.seeds.elevation_seed as u64);
        simplified_elevation_noise.set_noise_type(NoiseType::PerlinFractal);
        simplified_elevation_noise.set_fractal_type(FractalType::FBM);
        simplified_elevation_noise.set_fractal_octaves(3);
        simplified_elevation_noise.set_fractal_gain(0.4);
        simplified_elevation_noise.set_fractal_lacunarity(2.0);
        simplified_elevation_noise.set_frequency(2.0);
        let mut moisture_noise = FastNoise::seeded(self.seeds.moisture_seed as u64);
        moisture_noise.set_noise_type(NoiseType::PerlinFractal);
        moisture_noise.set_fractal_type(FractalType::FBM);
        moisture_noise.set_fractal_octaves(5);
        moisture_noise.set_fractal_gain(0.75);
        moisture_noise.set_fractal_lacunarity(2.0);
        moisture_noise.set_frequency(2.0);

        fn color_to_vec3(color: Color) -> Vec3 {
            Vec3::new(color.r(), color.g(), color.b())
        }
        let moisture_mountain = color_to_vec3(Color::SILVER);
        let moisture_prairie = color_to_vec3(Color::ORANGE_RED);
        let arid_mountain = color_to_vec3(Color::ORANGE);
        let arid_prairie = color_to_vec3(Color::SALMON);
        let mut simplified_vertices = Vec::new();
        let mut vertices = Vec::new();
        let mut colors = Vec::new();
        let mut metallic_roughness = Vec::new();
        let low = LOW_DEF as f32;
        let high = HIGH_DEF as f32;
        let error_margin = 1.0 / high / 2.0;

        let mut cached = CachedNoise::new(simplified_elevation_noise);
        for i in 0..=HIGH_DEF {
            for j in 0..=HIGH_DEF {
                let xz = (i as f32 / HIGH_DEF as f32, j as f32 / HIGH_DEF as f32);
                let xz_low = (
                    ((xz.0 * LOW_DEF as f32) as u32) as f32 / LOW_DEF as f32,
                    ((xz.1 * LOW_DEF as f32) as u32) as f32 / LOW_DEF as f32,
                );
                let nx = self.x + xz.0;
                let ny = self.y + xz.1;
                let nx_low = self.x + xz_low.0;
                let ny_low = self.y + xz_low.1;
                let moisture = moisture_noise.get_noise(nx, ny) + 0.5;
                let elevation = elevation_noise.get_noise(nx, ny);

                let ixz_low = ((xz.0 * low) as i32, (xz.1 * low) as i32);
                let elevation_block = cached.get_noise(nx_low, ny_low, ixz_low.0, ixz_low.1);
                let bottom = cached.get_noise(nx_low, ny_low + 1.0 / low, ixz_low.0, ixz_low.1 + 1);
                let top = cached.get_noise(nx_low, ny_low - 1.0 / low, ixz_low.0, ixz_low.1 - 1);
                let left = cached.get_noise(nx_low - 1.0 / low, ny_low, ixz_low.0 - 1, ixz_low.1);
                let right = cached.get_noise(nx_low + 1.0 / low, ny_low, ixz_low.0 + 1, ixz_low.1);
                let mut kind = 0;

                if Self::is_obstacle(elevation_block) {
                    if Self::is_obstacle(top) {
                        kind |= 1;
                    }
                    if Self::is_obstacle(bottom) {
                        kind |= 4;
                    }
                    if Self::is_obstacle(left) {
                        kind |= 2;
                    }
                    if Self::is_obstacle(right) {
                        kind |= 8;
                    }
                }

                let elevation_flattened = Self::pretty_border(
                    kind,
                    elevation_block,
                    (xz.0 - xz_low.0) * high - (HIGH_DEF / LOW_DEF / 2) as f32 + 0.5,
                    (xz.1 - xz_low.1) * high - (HIGH_DEF / LOW_DEF / 2) as f32 + 0.5,
                    elevation,
                );
                if (xz.0 - xz_low.0).abs() < error_margin && (xz.1 - xz_low.1).abs() < error_margin
                {
                    let simple_height = if kind > 0 {
                        Self::obstacle_height(elevation_block, elevation_block)
                    } else {
                        elevation_block / FLATTENING
                    };
                    simplified_vertices.push((
                        [xz.0 - 0.5, simple_height, xz.1 - 0.5],
                        [0.0, 0.0, 0.0],
                        [xz.1, xz.0],
                    ));
                }
                vertices.push((
                    [xz.0 - 0.5, elevation_flattened, xz.1 - 0.5],
                    [0.0, 0.0, 0.0],
                    [xz.1, xz.0],
                ));

                let elevation = elevation + 0.5;
                let mountain =
                    arid_mountain.lerp(moisture_mountain, (moisture * 2.0).clamp(0.0, 1.0));
                let prairie = arid_prairie.lerp(moisture_prairie, (moisture * 2.0).clamp(0.0, 1.0));
                let lerped = prairie.lerp(mountain, elevation);
                colors.extend_from_slice(&[
                    (lerped.x * 255.0) as u8,
                    (lerped.y * 255.0) as u8,
                    (lerped.z * 255.0) as u8,
                    255,
                ]);

                let roughness = ((1.0 - elevation) * 2.0).clamp(0.0, 1.0);
                let metallic = 1.0 - moisture;
                metallic_roughness.extend_from_slice(&[
                    0,
                    (roughness * 255.0) as u8,
                    (metallic * 255.0) as u8,
                    255,
                ])
            }
        }
        let mesh = vertices_as_mesh(vertices, HIGH_DEF);

        let simplified_mesh = vertices_as_mesh(simplified_vertices, LOW_DEF);

        Terrain {
            mesh,
            simplified_mesh,
            color: Image::new(
                Extent3d {
                    width: HIGH_DEF + 1,
                    height: HIGH_DEF + 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                colors,
                TextureFormat::Rgba8UnormSrgb,
            ),
            metallic_roughness: Image::new(
                Extent3d {
                    width: HIGH_DEF + 1,
                    height: HIGH_DEF + 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                metallic_roughness,
                TextureFormat::Rgba8UnormSrgb,
            ),
        }
    }
}

fn vertices_as_mesh(vertices: Vec<([f32; 3], [f32; 3], [f32; 2])>, details: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut indices = vec![];
    for i in 0..details {
        for j in 0..details {
            indices.extend_from_slice(&[
                i + (details + 1) * j,
                i + 1 + (details + 1) * j,
                i + (details + 1) * (j + 1),
            ]);
            indices.extend_from_slice(&[
                i + (details + 1) * (j + 1),
                i + 1 + (details + 1) * j,
                i + 1 + (details + 1) * (j + 1),
            ]);
        }
    }

    let mut indices_iter = indices.iter();
    while let Some(a) = indices_iter.next() {
        let b = indices_iter.next().unwrap();
        let c = indices_iter.next().unwrap();

        let pa = Vec3::from(positions[*a as usize]);
        let pb = Vec3::from(positions[*b as usize]);
        let pc = Vec3::from(positions[*c as usize]);

        let ab = pb - pa;
        let bc = pc - pb;
        let ca = pa - pc;
        let normal_face = ab.cross(bc) + bc.cross(ca) + ca.cross(ab);

        let na = Vec3::from(normals[*a as usize]);
        let nb = Vec3::from(normals[*b as usize]);
        let nc = Vec3::from(normals[*c as usize]);
        (na + normal_face).write_to_slice(&mut normals[*a as usize]);
        (nb + normal_face).write_to_slice(&mut normals[*b as usize]);
        (nc + normal_face).write_to_slice(&mut normals[*c as usize]);
    }

    let normals: Vec<_> = normals
        .into_iter()
        .map(|normal| {
            let normal = Vec3::from(normal);
            let normalized = normal.normalize();
            [normalized.x, normalized.y, normalized.z]
        })
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

pub struct Terrain {
    pub simplified_mesh: Mesh,
    pub mesh: Mesh,
    pub color: Image,
    pub metallic_roughness: Image,
}
