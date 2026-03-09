/*
 * Copyright (C) 2024 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use dae_parser::{Document, Geometry, GeometryElement, Primitive, Semantic, UpAxis};

use thiserror::Error;

pub struct DaePlugin;

impl Plugin for DaePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<DaeLoader>();
    }
}

#[derive(Default)]
struct DaeLoader;

impl AssetLoader for DaeLoader {
    type Asset = Mesh;
    type Settings = ();
    type Error = DaeError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes)?;
        load_dae_mesh(text)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["dae"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum DaeError {
    #[error("Couldn't read DAE file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("DAE parse error: {0}")]
    ParseError(String),
    #[error("No geometry found in DAE file")]
    NoGeometry,
}

fn load_dae_mesh(text: &str) -> Result<Mesh, DaeError> {
    let doc: Document = text.parse().map_err(|e: String| DaeError::ParseError(e))?;

    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    let mut all_normals: Vec<[f32; 3]> = Vec::new();
    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();
    let mut has_normals = false;
    let mut has_uvs = false;

    for geom in doc.iter::<Geometry>() {
        let mesh = match &geom.element {
            GeometryElement::Mesh(m) => m,
            _ => continue,
        };

        let maps = doc.local_maps();

        // Build source lookup: source id -> (float data, stride, offset)
        let Some(vertices) = &mesh.vertices else {
            continue;
        };

        for prim in &mesh.elements {
            let (inputs, prim_data) = match prim {
                Primitive::Triangles(tri) => {
                    let data = match &tri.data.prim {
                        Some(d) => d,
                        None => continue,
                    };
                    (&tri.inputs, PrimData::Flat(data))
                }
                Primitive::PolyList(poly) => (
                    &poly.inputs,
                    PrimData::PolyList {
                        vcount: &poly.data.vcount,
                        indices: &poly.data.prim,
                    },
                ),
                Primitive::TriFans(fans) => (&fans.inputs, PrimData::TriFan(&fans.data.prim)),
                Primitive::TriStrips(strips) => {
                    (&strips.inputs, PrimData::TriStrip(&strips.data.prim))
                }
                _ => continue,
            };

            let stride = inputs.stride;
            if stride == 0 {
                continue;
            }

            // Find which offset corresponds to which semantic
            let mut vertex_offset = None;
            let mut normal_offset = None;
            let mut texcoord_offset = None;

            for input in inputs.iter() {
                match input.semantic {
                    Semantic::Vertex => vertex_offset = Some(input.offset as usize),
                    Semantic::Normal => normal_offset = Some(input.offset as usize),
                    Semantic::TexCoord => {
                        if texcoord_offset.is_none() {
                            texcoord_offset = Some(input.offset as usize);
                        }
                    }
                    _ => {}
                }
            }

            let vertex_off = match vertex_offset {
                Some(o) => o,
                None => continue,
            };

            // Resolve position source from vertices
            let pos_input = vertices.position_input();
            let pos_source = match maps.get(pos_input.source_as_source()) {
                Some(s) => s,
                None => continue,
            };
            let pos_floats = match &pos_source.array {
                Some(dae_parser::ArrayElement::Float(arr)) => &arr.val,
                _ => continue,
            };
            let pos_stride = pos_source.accessor.stride;
            let pos_offset = pos_source.accessor.offset;

            // Resolve normal source if present
            let norm_floats = normal_offset.and_then(|_| {
                let norm_input = inputs.iter().find(|i| i.semantic == Semantic::Normal)?;
                let norm_source = maps.get(norm_input.source_as_source())?;
                match &norm_source.array {
                    Some(dae_parser::ArrayElement::Float(arr)) => Some((
                        &arr.val,
                        norm_source.accessor.stride,
                        norm_source.accessor.offset,
                    )),
                    _ => None,
                }
            });

            // Resolve texcoord source if present
            let uv_floats = texcoord_offset.and_then(|_| {
                let uv_input = inputs.iter().find(|i| i.semantic == Semantic::TexCoord)?;
                let uv_source = maps.get(uv_input.source_as_source())?;
                match &uv_source.array {
                    Some(dae_parser::ArrayElement::Float(arr)) => Some((
                        &arr.val,
                        uv_source.accessor.stride,
                        uv_source.accessor.offset,
                    )),
                    _ => None,
                }
            });

            // Convert primitives to triangles
            let triangles = match prim_data {
                PrimData::Flat(data) => data.to_vec(),
                PrimData::PolyList { vcount, indices } => {
                    triangulate_polylist(vcount, indices, stride)
                }
                PrimData::TriFan(fans) => {
                    let mut result = Vec::new();
                    for fan in fans.iter() {
                        let num_verts = fan.len() / stride;
                        if num_verts < 3 {
                            continue;
                        }
                        for i in 1..num_verts - 1 {
                            result.extend_from_slice(&fan[0..stride]);
                            result.extend_from_slice(&fan[i * stride..(i + 1) * stride]);
                            result.extend_from_slice(&fan[(i + 1) * stride..(i + 2) * stride]);
                        }
                    }
                    result
                }
                PrimData::TriStrip(strips) => {
                    let mut result = Vec::new();
                    for strip in strips.iter() {
                        let num_verts = strip.len() / stride;
                        if num_verts < 3 {
                            continue;
                        }
                        for i in 0..num_verts - 2 {
                            if i % 2 == 0 {
                                result.extend_from_slice(&strip[i * stride..(i + 1) * stride]);
                                result
                                    .extend_from_slice(&strip[(i + 1) * stride..(i + 2) * stride]);
                                result
                                    .extend_from_slice(&strip[(i + 2) * stride..(i + 3) * stride]);
                            } else {
                                result
                                    .extend_from_slice(&strip[(i + 1) * stride..(i + 2) * stride]);
                                result.extend_from_slice(&strip[i * stride..(i + 1) * stride]);
                                result
                                    .extend_from_slice(&strip[(i + 2) * stride..(i + 3) * stride]);
                            }
                        }
                    }
                    result
                }
            };

            // Now iterate triangle data and emit vertices
            let num_verts = triangles.len() / stride;
            let base_index = all_positions.len() as u32;

            for v in 0..num_verts {
                let base = v * stride;
                let pos_idx = triangles[base + vertex_off] as usize;
                let p_start = pos_offset + pos_idx * pos_stride;

                // COLLADA uses Y-up by default; Bevy also uses Y-up, so no conversion needed.
                // However, many Gazebo models use Z-up. We check the <up_axis> element.
                let px = pos_floats.get(p_start).copied().unwrap_or(0.0);
                let py = pos_floats.get(p_start + 1).copied().unwrap_or(0.0);
                let pz = pos_floats.get(p_start + 2).copied().unwrap_or(0.0);
                all_positions.push([px, py, pz]);

                if let (Some(norm_off), Some((norm_data, norm_stride, norm_base_offset))) =
                    (normal_offset, &norm_floats)
                {
                    let norm_idx = triangles[base + norm_off] as usize;
                    let n_start = norm_base_offset + norm_idx * norm_stride;
                    let nx = norm_data.get(n_start).copied().unwrap_or(0.0);
                    let ny = norm_data.get(n_start + 1).copied().unwrap_or(0.0);
                    let nz = norm_data.get(n_start + 2).copied().unwrap_or(0.0);
                    all_normals.push([nx, ny, nz]);
                    has_normals = true;
                } else if has_normals {
                    all_normals.push([0.0, 1.0, 0.0]);
                }

                if let (Some(uv_off), Some((uv_data, uv_stride, uv_base_offset))) =
                    (texcoord_offset, &uv_floats)
                {
                    let uv_idx = triangles[base + uv_off] as usize;
                    let t_start = uv_base_offset + uv_idx * uv_stride;
                    let u = uv_data.get(t_start).copied().unwrap_or(0.0);
                    let v_coord = uv_data.get(t_start + 1).copied().unwrap_or(0.0);
                    // COLLADA UVs: flip V for Bevy (OpenGL convention)
                    all_uvs.push([u, 1.0 - v_coord]);
                    has_uvs = true;
                } else if has_uvs {
                    all_uvs.push([0.0, 0.0]);
                }

                all_indices.push(base_index + v as u32);
            }
        }
    }

    if all_positions.is_empty() {
        return Err(DaeError::NoGeometry);
    }

    // Check up_axis and convert if Z_UP
    if doc.asset.up_axis == UpAxis::ZUp {
        // Convert Z-up to Y-up: (x, y, z) -> (x, z, -y)
        for pos in &mut all_positions {
            let [x, y, z] = *pos;
            *pos = [x, z, -y];
        }
        for norm in &mut all_normals {
            let [x, y, z] = *norm;
            *norm = [x, z, -y];
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, all_positions);

    if has_normals && !all_normals.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, all_normals);
    } else {
        mesh.compute_normals();
    }

    if has_uvs && !all_uvs.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs);
    }

    mesh.insert_indices(Indices::U32(all_indices));

    Ok(mesh)
}

enum PrimData<'a> {
    Flat(&'a [u32]),
    PolyList {
        vcount: &'a [u32],
        indices: &'a [u32],
    },
    TriFan(&'a [Box<[u32]>]),
    TriStrip(&'a [Box<[u32]>]),
}

/// Triangulate a polylist by fan triangulation of each polygon.
fn triangulate_polylist(vcount: &[u32], prim: &[u32], stride: usize) -> Vec<u32> {
    let mut result = Vec::new();
    let mut offset = 0usize;
    for &vc in vcount {
        let n = vc as usize;
        if n < 3 {
            offset += n * stride;
            continue;
        }
        // Fan triangulation: for polygon with vertices v0..v_{n-1},
        // emit triangles (v0, v1, v2), (v0, v2, v3), ...
        for i in 1..n - 1 {
            // v0
            result.extend_from_slice(&prim[offset..offset + stride]);
            // v_i
            let vi = offset + i * stride;
            result.extend_from_slice(&prim[vi..vi + stride]);
            // v_{i+1}
            let vi1 = offset + (i + 1) * stride;
            result.extend_from_slice(&prim[vi1..vi1 + stride]);
        }
        offset += n * stride;
    }
    result
}
