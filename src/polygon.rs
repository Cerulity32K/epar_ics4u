use std::{array::TryFromSliceError, io};

use macroquad::{
    color::Color,
    math::{Mat2, Vec2},
    shapes::draw_triangle,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolygonDecodeError {
    /*#[error("too many points while decoding polygon")]
    TooManyPoints,*/
    #[error("io error while decoding polygon")]
    IoError(#[from] io::Error),
    #[error("unreachable logic while decoding polygon")]
    SliceError(#[from] TryFromSliceError),
}

#[derive(Clone)]
pub struct Polygon {
    pub vertices: Vec<Vec2>,
    pub indices: Vec<usize>,
}
impl Polygon {
    /*pub const MAX_VERTICES: usize = 32768;
    pub const MAX_INDEX_TRIPLETS: usize = 32768;
    pub fn from_stream(reader: &mut impl Read) -> Result<Self, PolygonDecodeError> {
        let mut buf = [0u8; 8];

        reader.read(&mut buf)?;
        let vertex_count = usize::from_le_bytes(buf);
        if vertex_count > Self::MAX_VERTICES {
            return Err(PolygonDecodeError::TooManyPoints);
        }
        let mut vertices = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            reader.read(&mut buf)?;
            let xbuf: [u8; 4] = (&buf[..4]).try_into()?;
            let ybuf: [u8; 4] = (&buf[4..]).try_into()?;
            vertices.push(Vec2 {
                x: f32::from_le_bytes(xbuf),
                y: f32::from_le_bytes(ybuf),
            });
        }

        reader.read(&mut buf)?;
        let index_count = usize::from_le_bytes(buf);
        if index_count > Self::MAX_VERTICES {
            return Err(PolygonDecodeError::TooManyPoints);
        }
        let mut indices = Vec::with_capacity(index_count);
        for _ in 0..index_count {
            reader.read(&mut buf)?;
            indices.push(usize::from_le_bytes(buf));
        }
        Ok(Polygon { vertices, indices })
    }*/
    pub fn draw_mat(&self, color: Color, offset: Vec2, transformation: Mat2) {
        for [i1, i2, i3] in self
            .indices
            .chunks(3)
            .flat_map(|chunk| -> Option<[usize; 3]> { chunk.try_into().ok() })
        {
            if let (Some(v1), Some(v2), Some(v3)) = (
                self.vertices.get(i1),
                self.vertices.get(i2),
                self.vertices.get(i3),
            ) {
                draw_triangle(
                    transformation * *v1 + offset,
                    transformation * *v2 + offset,
                    transformation * *v3 + offset,
                    color,
                );
            }
        }
    }
    pub fn draw(&self, position: Vec2, rotation: f32, scale: Vec2, color: Color) {
        self.draw_mat(color, position, Mat2::from_scale_angle(scale, rotation));
    }
    /*pub fn from_slices_checked((vertices, indices): (&[Vec2], &[usize])) -> Option<Self> {
        if vertices.len() % 3 != 0 || indices.iter().any(|&i| i >= vertices.len()) {
            return None;
        }
        Some(Self::from_slices((vertices, indices)))
    }*/
    pub fn from_arrays<const V: usize, const I: usize>(
        (vertices, indices): ([Vec2; V], [usize; I]),
    ) -> Self {
        Self::from_slices((&vertices, &indices))
    }
    pub fn from_slices((vertices, indices): (&[Vec2], &[usize])) -> Self {
        Self {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
        }
    }
}

pub mod presets {
    use std::f32::consts::TAU as TAU32;
    use std::f64::consts::TAU as TAU64;

    use macroquad::math::Vec2;

    use crate::transform;

    pub fn generate_polygon<const N: usize>(rotation: f64) -> ([Vec2; N + 1], [usize; N * 3]) {
        let mut vertices = [Vec2::ZERO; N + 1];
        let mut indices = [0; N * 3];
        if N == 0 {
            return (vertices, indices);
        }
        let mut i = 0;
        while i < N {
            let period = i as f64 / N as f64 * TAU64 + rotation;
            vertices[i + 1] = Vec2::new((period.cos()) as f32, period.sin() as f32);
            indices[i * 3] = 0;
            indices[i * 3 + 1] = i + 1;
            indices[i * 3 + 2] = i + 2;
            i += 1;
        }
        indices[N * 3 - 1] = 1;
        return (vertices, indices);
    }

    pub fn generate_heart<const N: usize>() -> ([Vec2; N + 1], [usize; N * 3]) {
        let mut vertices = [Vec2::ZERO; N + 1];
        let mut indices = [0; N * 3];
        if N == 0 {
            return (vertices, indices);
        }
        let mut i = 0;
        while i < N {
            let period = i as f64 / N as f64 * TAU64;
            vertices[i + 1] = Vec2::new(
                period.sin() as f32,
                -(period.cos() + period.sin().abs() - 0.25) as f32 * 0.75,
            );
            indices[i * 3] = 0;
            indices[i * 3 + 1] = i + 1;
            indices[i * 3 + 2] = i + 2;
            i += 1;
        }
        indices[N * 3 - 1] = 1;
        return (vertices, indices);
    }
    pub fn generate_spokes<const N: usize>(
        outer_width: f32,
        inner_width: f32,
        rotation: f32,
    ) -> ([Vec2; N * 4], [usize; N * 6]) {
        let mut vertices = [Vec2::ZERO; N * 4];
        let mut indices = [0; N * 6];
        if N == 0 {
            return (vertices, indices);
        }
        let mut i = 0;

        let v1: Vec2 = Vec2::new(outer_width * 0.5, 1.0);
        let v2: Vec2 = Vec2::new(outer_width * -0.5, 1.0);
        let v3: Vec2 = Vec2::new(inner_width * 0.5, 0.0);
        let v4: Vec2 = Vec2::new(inner_width * -0.5, 0.0);

        while i < N {
            let period = i as f32 / N as f32 * TAU32 + rotation;

            vertices[i * 4 + 0] = transform::rotate(v1, period);
            vertices[i * 4 + 1] = transform::rotate(v2, period);
            vertices[i * 4 + 2] = transform::rotate(v3, period);
            vertices[i * 4 + 3] = transform::rotate(v4, period);

            indices[i * 6 + 0] = i * 4;
            indices[i * 6 + 1] = i * 4 + 1;
            indices[i * 6 + 2] = i * 4 + 2;
            indices[i * 6 + 3] = i * 4 + 3;
            indices[i * 6 + 4] = i * 4 + 1;
            indices[i * 6 + 5] = i * 4 + 2;

            i += 1;
        }
        println!("{vertices:?}\n{indices:?}");
        return (vertices, indices);
    }
    pub fn none<V, I>() -> ([V; 0], [I; 0]) {
        ([], [])
    }
}
