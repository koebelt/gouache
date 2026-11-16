//! WGSL library with `#import "file.wgsl"` preprocessing and mtime-based hot reload.
//!
//! Shaders live in `assets/shaders/`. Each file may contain lines of the form
//! `#import "common.wgsl"`; imports are inlined once (include-guard semantics),
//! depth first. Release builds still read from disk; a packaged build will embed
//! the directory (ADR-004).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::RenderError;

/// Loads and preprocesses WGSL sources and tracks their modification times.
pub struct ShaderLibrary {
    root: PathBuf,
    /// Source path -> mtime seen at last load (BTreeMap: deterministic order).
    seen: BTreeMap<PathBuf, SystemTime>,
}

impl ShaderLibrary {
    /// Default shader directory: `$GOUACHE_SHADERS` or `<repo>/assets/shaders`.
    pub fn default_root() -> PathBuf {
        std::env::var_os("GOUACHE_SHADERS")
            .map(PathBuf::from)
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/shaders"))
    }

    /// Library rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into(), seen: BTreeMap::new() }
    }

    /// Fully preprocessed source for `name` (relative to the root).
    pub fn source(&mut self, name: &str) -> Result<String, RenderError> {
        let mut out = String::new();
        let mut included = Vec::new();
        self.expand(name, &mut out, &mut included, 0)?;
        Ok(out)
    }

    fn expand(
        &mut self,
        name: &str,
        out: &mut String,
        included: &mut Vec<String>,
        depth: u32,
    ) -> Result<(), RenderError> {
        if depth > 16 {
            return Err(RenderError::Shader(format!("import depth exceeded at {name}")));
        }
        if included.iter().any(|n| n == name) {
            return Ok(());
        }
        included.push(name.to_string());
        let path = self.root.join(name);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| RenderError::Shader(format!("{}: {e}", path.display())))?;
        if let Ok(m) = std::fs::metadata(&path).and_then(|m| m.modified()) {
            self.seen.insert(path.clone(), m);
        }
        for line in text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("#import") {
                let dep = rest.trim().trim_matches('"');
                self.expand(dep, out, included, depth + 1)?;
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        Ok(())
    }

    /// True if any loaded file changed on disk since it was last read.
    pub fn changed(&self) -> bool {
        self.seen
            .iter()
            .any(|(p, t)| std::fs::metadata(p).and_then(|m| m.modified()).map(|m| m != *t).unwrap_or(false))
    }

    /// Forget mtimes (call before a full rebuild).
    pub fn reset(&mut self) {
        self.seen.clear();
    }

    /// Compile a module, surfacing validation errors as `RenderError::Shader`.
    pub fn module(&mut self, device: &wgpu::Device, name: &str) -> Result<wgpu::ShaderModule, RenderError> {
        let src = self.source(name)?;
        // Validate with naga up front so errors carry file context instead of a device panic.
        let parsed = naga::front::wgsl::parse_str(&src)
            .map_err(|e| RenderError::Shader(format!("{name}: {}", e.emit_to_string(&src))))?;
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&parsed)
            .map_err(|e| RenderError::Shader(format!("{name}: {}", e.emit_to_string(&src))))?;
        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_once() {
        let dir = std::env::temp_dir().join(format!("gouache_shader_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.wgsl"), "const A: f32 = 1.0;\n").unwrap();
        std::fs::write(dir.join("b.wgsl"), "#import \"a.wgsl\"\nconst B: f32 = A;\n").unwrap();
        std::fs::write(dir.join("m.wgsl"), "#import \"a.wgsl\"\n#import \"b.wgsl\"\n").unwrap();
        let mut lib = ShaderLibrary::new(&dir);
        let s = lib.source("m.wgsl").unwrap();
        assert_eq!(s.matches("const A").count(), 1);
        assert!(s.contains("const B"));
        assert!(!lib.changed());
    }
}
