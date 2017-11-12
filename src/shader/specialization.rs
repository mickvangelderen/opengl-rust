extern crate core;
extern crate gl;

use gl::types::GLuint;
use core::marker::PhantomData;

pub trait ShaderKindMarker {
    const VALUE: super::ShaderKind;
}

macro_rules! impl_kind {
    ($Kind:ident, $Value:path) => {
        #[derive(Debug)]
        pub struct $Kind;

        impl ShaderKindMarker for $Kind {
            const VALUE: super::ShaderKind = $Value;
        }
    }
}

impl_kind!(ComputeShaderKind, super::ShaderKind::Compute);
impl_kind!(FragmentShaderKind, super::ShaderKind::Fragment);
impl_kind!(GeometryShaderKind, super::ShaderKind::Geometry);
impl_kind!(VertexShaderKind, super::ShaderKind::Vertex);
impl_kind!(
    TesselationControlShaderKind,
    super::ShaderKind::TesselationControl
);
impl_kind!(
    TesselationEvaluationShaderKind,
    super::ShaderKind::TesselationEvaluation
);

#[derive(Debug)]
pub struct ShaderId<Kind: ShaderKindMarker>(super::ShaderId, PhantomData<Kind>);

impl<Kind: ShaderKindMarker> ShaderId<Kind> {
    #[inline]
    pub fn new() -> Option<Self> {
        super::ShaderId::new(Kind::VALUE).map(|id| ShaderId(id, PhantomData))
    }

    #[inline]
    pub fn compile<T: AsRef<str>>(self, sources: &[T]) -> Result<CompiledShaderId<Kind>, String> {
        self.0.compile(sources).map(|id| {
            CompiledShaderId(id, PhantomData)
        })
    }

    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.as_uint()
    }
}

impl<Kind: ShaderKindMarker> AsRef<super::ShaderId> for ShaderId<Kind> {
    #[inline]
    fn as_ref(&self) -> &super::ShaderId {
        &self.0
    }
}

impl<Kind: ShaderKindMarker> From<ShaderId<Kind>> for super::ShaderId {
    #[inline]
    fn from(value: ShaderId<Kind>) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct CompiledShaderId<Kind: ShaderKindMarker>(super::CompiledShaderId, PhantomData<Kind>);

impl<Kind: ShaderKindMarker> CompiledShaderId<Kind> {
    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.as_uint()
    }
}

impl<Kind: ShaderKindMarker> AsRef<super::CompiledShaderId> for CompiledShaderId<Kind> {
    #[inline]
    fn as_ref(&self) -> &super::CompiledShaderId {
        &self.0
    }
}

impl<Kind: ShaderKindMarker> From<CompiledShaderId<Kind>> for super::CompiledShaderId {
    #[inline]
    fn from(value: CompiledShaderId<Kind>) -> Self {
        value.0
    }
}

pub type ComputeShaderId = ShaderId<ComputeShaderKind>;
pub type GeometryShaderId = ShaderId<GeometryShaderKind>;
pub type FragmentShaderId = ShaderId<FragmentShaderKind>;
pub type VertexShaderId = ShaderId<VertexShaderKind>;
pub type TesselationControlShaderId = ShaderId<TesselationControlShaderKind>;
pub type TesselationEvaluationShaderId = ShaderId<TesselationEvaluationShaderKind>;

pub type CompiledComputeShaderId = CompiledShaderId<ComputeShaderKind>;
pub type CompiledFragmentShaderId = CompiledShaderId<FragmentShaderKind>;
pub type CompiledGeometryShaderId = CompiledShaderId<GeometryShaderKind>;
pub type CompiledVertexShaderId = CompiledShaderId<VertexShaderKind>;
pub type CompiledTesselationControlShaderId = CompiledShaderId<TesselationControlShaderKind>;
pub type CompiledTesselationEvaluationShaderId = CompiledShaderId<TesselationEvaluationShaderKind>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn as_ref_example() {
        let vs_id = VertexShaderId::new()
            .unwrap()
            .compile(&[ String::default() ])
            .unwrap();

        let fs_id = FragmentShaderId::new()
            .unwrap()
            .compile(&[ String::default() ])
            .unwrap();

        let _ids: [&super::super::CompiledShaderId; 2] = [vs_id.as_ref(), fs_id.as_ref()];
    }
}
