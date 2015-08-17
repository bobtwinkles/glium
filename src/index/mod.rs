/*!
In order to draw, you need to provide a way for the video card to know how to link primitives
together.

There are eleven types of primitives, each one with a corresponding struct:

 - `PointsList`
 - `LinesList`
 - `LinesListAdjacency`
 - `LineStrip`
 - `LineStripAdjacency`
 - `TrianglesList`
 - `TrianglesListAdjacency`
 - `TriangleStrip`
 - `TriangleStripAdjacency`
 - `TriangleFan`
 - `Patches`

There are two ways to specify the indices that must be used:

 - Passing a reference to an `IndexBuffer`, which contains a list of indices.
 - `NoIndices`, in which case the vertices will be used in the order in which they are in the
   vertex buffer.

## Multidraw indirect

In addition to indices, you can also use **multidraw indirect** rendering.

The idea is to put a list of things to render in a buffer, and pass that buffer to OpenGL.

*/
use gl;
use ToGlEnum;
use CapabilitiesSource;
use version::Api;
use version::Version;

use std::mem;

use buffer::BufferAnySlice;

pub use self::buffer::{IndexBuffer, IndexBufferSlice, IndexBufferAny};
pub use self::buffer::CreationError as BufferCreationError;
pub use self::multidraw::{DrawCommandsNoIndicesBuffer, DrawCommandNoIndices};
pub use self::multidraw::{DrawCommandsIndicesBuffer, DrawCommandIndices};

mod buffer;
mod multidraw;

/// Describes a source of indices used for drawing.
#[derive(Clone)]
pub enum IndicesSource<'a> {
    /// A buffer uploaded in video memory.
    IndexBuffer {
        /// The buffer.
        buffer: BufferAnySlice<'a>,
        /// Type of indices in the buffer.
        data_type: IndexType,
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },

    /// Use a multidraw indirect buffer without indices.
    MultidrawArray {
        /// The buffer.
        buffer: BufferAnySlice<'a>,
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },

    /// Use a multidraw indirect buffer with indices.
    MultidrawElement {
        /// The buffer of the commands.
        commands: BufferAnySlice<'a>,
        /// The buffer of the indices.
        indices: BufferAnySlice<'a>,
        /// Type of indices in the buffer.
        data_type: IndexType,
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },

    /// Don't use indices. Assemble primitives by using the order in which the vertices are in
    /// the vertices source.
    NoIndices {
        /// Type of primitives contained in the vertex source.
        primitives: PrimitiveType,
    },
}

impl<'a> IndicesSource<'a> {
    /// Returns the type of the primitives.
    #[inline]
    pub fn get_primitives_type(&self) -> PrimitiveType {
        match self {
            &IndicesSource::IndexBuffer { primitives, .. } => primitives,
            &IndicesSource::MultidrawArray { primitives, .. } => primitives,
            &IndicesSource::MultidrawElement { primitives, .. } => primitives,
            &IndicesSource::NoIndices { primitives } => primitives,
        }
    }
}

/// List of available primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    ///
    Points,
    ///
    LinesList,
    ///
    LinesListAdjacency,
    ///
    LineStrip,
    ///
    LineStripAdjacency,
    ///
    LineLoop,
    ///
    TrianglesList,
    ///
    TrianglesListAdjacency,
    ///
    TriangleStrip,
    ///
    TriangleStripAdjacency,
    ///
    TriangleFan,
    ///
    Patches {
        /// Number of vertices per patch.
        vertices_per_patch: u16,
    },
}

impl PrimitiveType {
    /// Returns true if the backend supports this type of primitives.
    pub fn is_supported<C>(&self, caps: &C) -> bool where C: CapabilitiesSource {
        match self {
            &PrimitiveType::Points | &PrimitiveType::LinesList | &PrimitiveType::LineStrip |
            &PrimitiveType::LineLoop | &PrimitiveType::TrianglesList |
            &PrimitiveType::TriangleStrip | &PrimitiveType::TriangleFan => true,

            &PrimitiveType::LinesListAdjacency | &PrimitiveType::LineStripAdjacency |
            &PrimitiveType::TrianglesListAdjacency | &PrimitiveType::TriangleStripAdjacency => {
                caps.get_version() >= &Version(Api::Gl, 3, 0) ||
                caps.get_extensions().gl_arb_geometry_shader4 ||
                caps.get_extensions().gl_ext_geometry_shader4 ||
                caps.get_extensions().gl_ext_geometry_shader
            },

            &PrimitiveType::Patches { .. } => {
                caps.get_version() >= &Version(Api::Gl, 4, 0) ||
                caps.get_extensions().gl_arb_tessellation_shader
            },
        }
    }
}

impl ToGlEnum for PrimitiveType {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        match self {
            &PrimitiveType::Points => gl::POINTS,
            &PrimitiveType::LinesList => gl::LINES,
            &PrimitiveType::LinesListAdjacency => gl::LINES_ADJACENCY,
            &PrimitiveType::LineStrip => gl::LINE_STRIP,
            &PrimitiveType::LineStripAdjacency => gl::LINE_STRIP_ADJACENCY,
            &PrimitiveType::LineLoop => gl::LINE_LOOP,
            &PrimitiveType::TrianglesList => gl::TRIANGLES,
            &PrimitiveType::TrianglesListAdjacency => gl::TRIANGLES_ADJACENCY,
            &PrimitiveType::TriangleStrip => gl::TRIANGLE_STRIP,
            &PrimitiveType::TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
            &PrimitiveType::TriangleFan => gl::TRIANGLE_FAN,
            &PrimitiveType::Patches { .. } => gl::PATCHES,
        }
    }
}

/// Marker that can be used as an indices source when you don't need indices.
///
/// If you use this, then the primitives will be constructed using the order in which the
/// vertices are in the vertices sources.
#[derive(Copy, Clone, Debug)]
pub struct NoIndices(pub PrimitiveType);

impl<'a> From<NoIndices> for IndicesSource<'a> {
    #[inline]
    fn from(marker: NoIndices) -> IndicesSource<'a> {
        IndicesSource::NoIndices {
            primitives: marker.0
        }
    }
}

impl<'a, 'b> From<&'b NoIndices> for IndicesSource<'a> {
    #[inline]
    fn from(marker: &'b NoIndices) -> IndicesSource<'a> {
        IndicesSource::NoIndices {
            primitives: marker.0
        }
    }
}

/// Type of the indices in an index source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]    // GLenum
pub enum IndexType {
    /// u8
    U8 = gl::UNSIGNED_BYTE,
    /// u16
    U16 = gl::UNSIGNED_SHORT,
    /// u32
    U32 = gl::UNSIGNED_INT,
}

impl IndexType {
    /// Returns the size in bytes of each index of this type.
    #[inline]
    pub fn get_size(&self) -> usize {
        match *self {
            IndexType::U8 => mem::size_of::<u8>(),
            IndexType::U16 => mem::size_of::<u16>(),
            IndexType::U32 => mem::size_of::<u32>(),
        }
    }

    /// Returns true if the backend supports this type of index.
    #[inline]
    pub fn is_supported<C>(&self, caps: &C) -> bool where C: CapabilitiesSource {
        match self {
            &IndexType::U8 => true,
            &IndexType::U16 => true,
            &IndexType::U32 => {
                caps.get_version() >= &Version(Api::Gl, 1, 0) ||
                caps.get_version() >= &Version(Api::GlEs, 3, 0)
            },
        }
    }
}

impl ToGlEnum for IndexType {
    #[inline]
    fn to_glenum(&self) -> gl::types::GLenum {
        *self as gl::types::GLenum
    }
}

/// An index from the index buffer.
pub unsafe trait Index: Copy + Send + 'static {
    /// Returns the `IndexType` corresponding to this type.
    fn get_type() -> IndexType;

    /// Returns true if this type of index is supported by the backend.
    fn is_supported<C>(caps: &C) -> bool where C: CapabilitiesSource {
        Self::get_type().is_supported(caps)
    }
}

unsafe impl Index for u8 {
    #[inline]
    fn get_type() -> IndexType {
        IndexType::U8
    }
}

unsafe impl Index for u16 {
    #[inline]
    fn get_type() -> IndexType {
        IndexType::U16
    }
}

unsafe impl Index for u32 {
    #[inline]
    fn get_type() -> IndexType {
        IndexType::U32
    }
}
