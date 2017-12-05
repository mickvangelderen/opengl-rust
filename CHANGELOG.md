# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Link to 0.1.0 in changelog.
- `bind` method to VertexArray.
- Derived `Debug` implementations.
- `BufferTarget` enum.
- `bind` method to VertexBuffer.
- Crate model with diffuse and specular map.
- Light with ambient, diffuse and specular color in fragment shader.
- Material with diffuse texture, specular texture and shininess in fragment shader.
- Attenuation to point light.

### Changed
- `ProgramId::attach` is now performed by `ProgramId::link`.
- Renamed `ProgramId::use_program` to `ProgramId::bind`.
- `ShaderId::compile` and friends now take `sources: &[&str]` as a parameter
  instead of `sources: &[T]` where `T: AsRef<str>`. The old signature would not
  allow `compile(&[a, b])` where `a` and `b` have different types that both do
  implement `AsRef<str>`, requiring the caller to type `compile(&[a.as_ref(),
  b.as_ref()])`. If that is a common case, we might as well have the parameter
  be `&[&str]` to keep things simple.

## [0.1.0] - 2017-11-25
### Added
- Changelog.
- Basic OpenGL rendering.
- Wavefront object file importing.
- A few toy models.

[Unreleased]: https://github.com/mickvangelderen/opengl-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mickvangelderen/opengl-rust/tree/v0.1.0
