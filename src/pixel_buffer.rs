/*!
Pixel buffers are buffers that contain two-dimensional texture data.

Contrary to textures, pixel buffers are stored in a client-defined format. They are used
to transfer data to or from the video memory, before or after being turned into a texture.
 */
use std::borrow::Cow;
use std::marker::PhantomData;

use backend::Facade;

use texture::{RawImage2d, Texture2dDataSink, ClientFormat};

use GlObject;
use BufferViewExt;
use buffer::{BufferView, BufferViewAny, BufferType};
use gl;

/// Buffer that stores the content of a texture.
///
/// The generic type represents the type of pixels that the buffer contains.
pub struct PixelBuffer<T> {
    buffer: BufferViewAny,
    dimensions: Option<(u32, u32)>,
    format: Option<ClientFormat>,
    marker: PhantomData<T>,
}

impl<T> PixelBuffer<T> {
    /// Builds a new buffer with an uninitialized content.
    pub fn new_empty<F>(facade: &F, capacity: usize) -> PixelBuffer<T> where F: Facade {
        PixelBuffer {
            buffer: BufferView::<u8>::empty(facade, BufferType::PixelPackBuffer, capacity,
                                           false).unwrap().into(),
            dimensions: None,
            format: None,
            marker: PhantomData,
        }
    }

    /// Returns the size of the buffer, in bytes.
    pub fn get_size(&self) -> usize {
        self.buffer.get_size()
    }
}

impl<T> PixelBuffer<T> where T: Texture2dDataSink {
    /// Copies the content of the pixel buffer to RAM.
    ///
    /// This operation is slow and should be done outside of the rendering loop.
    ///
    /// ## Panic
    ///
    /// Panics if the pixel buffer is empty.
    ///
    /// ## Features
    ///
    /// This function is only available if the `gl_read_buffer` feature is enabled.
    /// Otherwise, you should use `read_if_supported`.
    #[cfg(feature = "gl_read_buffer")]
    pub fn read(&self) -> T {
        self.read_if_supported().unwrap()
    }

    /// Copies the content of the pixel buffer to RAM.
    ///
    /// This operation is slow and should be done outside of the rendering loop.
    ///
    /// ## Panic
    ///
    /// Panics if the pixel buffer is empty.
    pub fn read_if_supported(&self) -> Option<T> {
        let data = match unsafe { self.buffer.read_if_supported() } {
            Some(d) => d,
            None => return None
        };

        let dimensions = self.dimensions.expect("The pixel buffer is empty");

        let data = RawImage2d {
            data: Cow::Owned(data),
            width: dimensions.0,
            height: dimensions.1,
            format: self.format.expect("The pixel buffer is empty"),
        };

        Some(Texture2dDataSink::from_raw(data))
    }
}

// TODO: rework this
impl<T> GlObject for PixelBuffer<T> {
    type Id = gl::types::GLuint;
    fn get_id(&self) -> gl::types::GLuint {
        self.buffer.get_buffer_id()
    }
}

// TODO: remove this hack
#[doc(hidden)]
pub fn store_infos<T>(b: &mut PixelBuffer<T>, dimensions: (u32, u32), format: ClientFormat) {
    b.dimensions = Some(dimensions);
    b.format = Some(format);
}
