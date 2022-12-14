use bevy::{
    math::Vec3,
    prelude::{Color, IVec2, Image, Mesh, Vec2},
    render::{
        mesh::Indices,
        render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    },
    utils::{Entry, HashMap},
};
use bracket_noise::prelude::*;
use tracing::instrument;

use crate::game::terra::Plane;

const WATER_LEVEL: f32 = -10.0;
const PLATEAU_LEVEL: f32 = 0.25;
pub(crate) const LOW_DEF: u32 = 5;
pub(crate) const HIGH_DEF: u32 = 40;
const FLATTENING: f32 = 9.0;

pub(crate) struct HeightMap {
    seeds: crate::game::terra::TerraNoises,
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
    pub(crate) fn build_heightmap(
        x: f32,
        y: f32,
        plane: Plane,
        seeds: crate::game::terra::TerraNoises,
    ) -> Self {
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

    fn get_noises(seed: u64) -> (FastNoise, FastNoise) {
        let mut noise = FastNoise::seeded(seed);
        noise.set_noise_type(NoiseType::PerlinFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(7);
        noise.set_fractal_gain(0.4);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(2.0);
        let mut simplified_noise = FastNoise::seeded(seed);
        simplified_noise.set_noise_type(NoiseType::PerlinFractal);
        simplified_noise.set_fractal_type(FractalType::FBM);
        simplified_noise.set_fractal_octaves(3);
        simplified_noise.set_fractal_gain(0.4);
        simplified_noise.set_fractal_lacunarity(2.0);
        simplified_noise.set_frequency(2.0);
        (noise, simplified_noise)
    }

    #[instrument(skip(self))]
    pub(crate) fn into_mesh_and_texture(self) -> Terrain {
        let (material_elevation_noise, material_simplified_elevation_noise) =
            Self::get_noises(self.seeds.material_seed as u64);

        fn color_to_vec3(color: Color) -> Vec3 {
            Vec3::new(color.r(), color.g(), color.b())
        }
        let material_mountains = color_to_vec3(Color::hex("FFFAFA").unwrap());
        let material_plains = color_to_vec3(Color::hex("A3C058").unwrap());
        let ethereal_mountains = color_to_vec3(Color::VIOLET);
        let ethereal_plains = color_to_vec3(Color::ORANGE_RED);
        let low = LOW_DEF as f32;
        let high = HIGH_DEF as f32;
        let error_margin = 1.0 / high / 2.0;

        #[allow(clippy::type_complexity)]
        let generate = |noise: FastNoise,
                        simplified_noise: FastNoise|
         -> (
            Vec<[f32; 3]>, // vertices
            Vec<[f32; 3]>, // normals
            Vec<[f32; 2]>, // uvs
            Vec<IVec2>,    // simplified map
            Vec<u8>,       // colors material
            Vec<u8>,       // colors ethereal
            Vec<u8>,       // metallic_roughness
        ) {
            // let mut simplified_vertices = Vec::with_capacity(LOW_DEF as usize * LOW_DEF as usize);
            let mut map = Vec::with_capacity(LOW_DEF as usize * LOW_DEF as usize);
            let mut vertices = Vec::with_capacity(HIGH_DEF as usize * HIGH_DEF as usize);
            let mut normals = Vec::with_capacity(HIGH_DEF as usize * HIGH_DEF as usize);
            let mut uvs = Vec::with_capacity(HIGH_DEF as usize * HIGH_DEF as usize);
            let mut colors_material = Vec::with_capacity(LOW_DEF as usize * LOW_DEF as usize);
            let mut colors_ethereal = Vec::with_capacity(LOW_DEF as usize * LOW_DEF as usize);
            let mut metallic_roughness = Vec::with_capacity(LOW_DEF as usize * LOW_DEF as usize);
            let mut cached = CachedNoise::new(simplified_noise);
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
                    let elevation = noise.get_noise(nx, ny);

                    let mut elevation_for_block = |x: f32, z: f32, ix: i32, iz: i32| {
                        cached.get_noise(x, z, ix, iz)
                            + Vec2::ZERO.distance_squared(Vec2::new(x, z)) / 500.0
                            - 0.075
                    };

                    let ixz_low = ((xz.0 * low) as i32, (xz.1 * low) as i32);
                    let elevation_block = elevation_for_block(nx_low, ny_low, ixz_low.0, ixz_low.1);
                    let bottom =
                        elevation_for_block(nx_low, ny_low + 1.0 / low, ixz_low.0, ixz_low.1 + 1);
                    let top =
                        elevation_for_block(nx_low, ny_low - 1.0 / low, ixz_low.0, ixz_low.1 - 1);
                    let left =
                        elevation_for_block(nx_low - 1.0 / low, ny_low, ixz_low.0 - 1, ixz_low.1);
                    let right =
                        elevation_for_block(nx_low + 1.0 / low, ny_low, ixz_low.0 + 1, ixz_low.1);
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
                    if (xz.0 - xz_low.0).abs() < error_margin
                        && (xz.1 - xz_low.1).abs() < error_margin
                    {
                        let simple_height = if kind > 0 {
                            Self::obstacle_height(elevation_block, elevation_block)
                        } else {
                            elevation_block / FLATTENING
                        };
                        if simple_height > PLATEAU_LEVEL {
                            map.push(IVec2::new(
                                (xz.0 * LOW_DEF as f32) as i32,
                                (xz.1 * LOW_DEF as f32) as i32,
                            ));
                        }
                    }
                    vertices.push([xz.0 - 0.5, elevation_flattened, xz.1 - 0.5]);
                    normals.push([0.0, 0.0, 0.0]);
                    uvs.push([xz.1, xz.0]);

                    if (xz.0 - xz_low.0).abs() < error_margin
                        && (xz.1 - xz_low.1).abs() < error_margin
                    {
                        let elevation = elevation + 0.3;
                        let lerped = material_plains.lerp(material_mountains, elevation);
                        colors_material.extend_from_slice(&[
                            (lerped.x * 255.0) as u8,
                            (lerped.y * 255.0) as u8,
                            (lerped.z * 255.0) as u8,
                            255,
                        ]);
                        let lerped = ethereal_plains.lerp(ethereal_mountains, elevation);
                        colors_ethereal.extend_from_slice(&[
                            (lerped.x * 255.0) as u8,
                            (lerped.y * 255.0) as u8,
                            (lerped.z * 255.0) as u8,
                            255,
                        ]);

                        let roughness = ((1.0 - elevation) * 2.0).clamp(0.0, 1.0);
                        metallic_roughness.extend_from_slice(&[
                            0,
                            (roughness * 255.0) as u8,
                            (0.0 * 255.0) as u8,
                            255,
                        ]);
                    }
                }
            }
            (
                vertices,
                normals,
                uvs,
                map,
                colors_material,
                colors_ethereal,
                metallic_roughness,
            )
        };

        let (
            positions,
            normals,
            uvs,
            simplified_map,
            material_colors,
            ethereal_colors,
            metallic_roughness,
        ) = generate(
            material_elevation_noise,
            material_simplified_elevation_noise,
        );
        let mesh = vertices_as_mesh(positions, normals, uvs, HIGH_DEF);

        Terrain {
            mesh,
            simplified_map,
            material_color: Image::new(
                Extent3d {
                    width: LOW_DEF + 1,
                    height: LOW_DEF + 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                material_colors,
                TextureFormat::Rgba8UnormSrgb,
            ),
            ethereal_color: Image::new(
                Extent3d {
                    width: LOW_DEF + 1,
                    height: LOW_DEF + 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                ethereal_colors,
                TextureFormat::Rgba8UnormSrgb,
            ),
            metallic_roughness: Image::new(
                Extent3d {
                    width: LOW_DEF + 1,
                    height: LOW_DEF + 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                metallic_roughness,
                TextureFormat::Rgba8UnormSrgb,
            ),
        }
    }
}

fn vertices_as_mesh(
    positions: Vec<[f32; 3]>,
    mut normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    details: u32,
) -> Mesh {
    let mut indices = Vec::with_capacity(details as usize * details as usize * 6);
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

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    if !normals.is_empty() {
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
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_indices(Some(Indices::U32(indices)));

    if !uvs.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    mesh
}

pub(crate) struct Terrain {
    pub(crate) simplified_map: Vec<IVec2>,
    pub(crate) mesh: Mesh,
    pub(crate) material_color: Image,
    pub(crate) ethereal_color: Image,
    pub(crate) metallic_roughness: Image,
}
