extern crate gl;

use gl::types::*;

#[derive(Debug)]
pub struct Viewport {
    x: GLint,
    y: GLint,
    width: GLsizei,
    height: GLsizei,
}

impl Viewport {
    #[inline]
    pub fn new(width: GLsizei, height: GLsizei) -> Self {
        Viewport { x: 0, y: 0, width, height }
    }

    #[inline]
    pub fn with_position(x: GLint, y: GLint, width: GLsizei, height: GLsizei) -> Self {
        Viewport { x, y, width, height }
    }

    #[inline]
    pub fn update(&mut self) -> ViewportUpdate {
        ViewportUpdate(self)
    }

    #[inline]
    pub fn x(&self) -> GLint {
        self.x
    }

    #[inline]
    pub fn y(&self) -> GLint {
        self.y
    }

    #[inline]
    pub fn width(&self) -> GLsizei {
        self.width
    }

    #[inline]
    pub fn height(&self) -> GLsizei {
        self.height
    }

    #[inline]
    pub fn aspect(&self) -> f32 {
        (self.width as f32 / self.height as f32).abs()
    }
}

#[derive(Debug)]
pub struct ViewportUpdate<'a>(&'a mut Viewport);

impl<'a> ViewportUpdate<'a> {
    #[inline]
    pub fn x(&mut self, x: GLint) -> &mut Self {
        self.0.x = x;
        self
    }

    #[inline]
    pub fn y(&mut self, y: GLint) -> &mut Self {
        self.0.y = y;
        self
    }

    #[inline]
    pub fn width(&mut self, width: GLsizei) -> &mut Self {
        self.0.width = width;
        self
    }

    #[inline]
    pub fn height(&mut self, height: GLsizei) -> &mut Self {
        self.0.height = height;
        self
    }
}

impl<'a> Drop for ViewportUpdate<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::Viewport(self.0.x, self.0.y, self.0.width, self.0.height);
        }
    }
}
