use super::super::internal::*;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use std::collections::HashMap;

#[derive(Default, serde::Serialize)]
struct ShaderSourceContext {
    bindings: Vec<String>,
    declarations: Vec<String>,
}

pub struct ShaderSourceBuilder {
    tt: handlebars::Handlebars<'static>,
    ctx: ShaderSourceContext,
}

impl ShaderSourceBuilder {
    pub fn new() -> Self {
        let mut tt = handlebars::Handlebars::new();
        tt.register_escape_fn(handlebars::no_escape);
        tt.register_helper("add", Box::new(add_helper));
        Self {
            tt,
            ctx: ShaderSourceContext::default(),
        }
    }

    pub fn source(&mut self, template: &str) {
        self.tt
            .register_template_string("shader", template)
            .unwrap();
    }

    pub fn mixin(&mut self, content: &str) {
        let sections = split_sections(content);

        for (name, set) in sections {
            for content in set {
                match name.as_str() {
                    "comment" => {}
                    "binding" => self.ctx.bindings.push(content),
                    "declaration" => self.ctx.declarations.push(content),
                    _ => eprintln!("Unknown section: {}", name),
                }
            }
        }
    }

    pub fn build(&self, name: &str) -> String {
        let source = self.tt.render("shader", &self.ctx).unwrap();
        source
    }

    pub fn log_to_file(&self, shader_name: &str, source: &str) {
        use std::fs;
        use std::path::PathBuf;

        let mut path = PathBuf::from("runtime/shaders");
        fs::create_dir_all(&path).unwrap();
        path.push(format!("{}.wgsl", shader_name));
        fs::write(&path, source).unwrap();
    }
}

//=======================================================================//
// Template rendering helpers
//=======================================================================//

fn add_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(0);
    let b = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(0);
    out.write(&(a + b).to_string())?;
    Ok(())
}

//=======================================================================//
// Internals
//=======================================================================//

/// Splits a file into a map of Vec<String> by section names.
///
/// ```text
/// [[section1]]
/// This is some content.
///
/// [[section2]]
/// This is some more content.
///
/// [[section1]]
/// Multiple sections of the same name are allowed.
/// ```
fn split_sections(input: &str) -> HashMap<String, Vec<String>> {
    let mut sections = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_content = Vec::new();

    for line in input.lines() {
        if let Some(name) = line.strip_prefix("[[").and_then(|l| l.strip_suffix("]]")) {
            let name = name.trim().to_string();
            if let Some(name) = current_name.take() {
                sections
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .push(current_content.join("\n").trim().to_string());
            }
            current_name = Some(name);
            current_content = Vec::new();
        } else {
            current_content.push(line.to_string());
        }
    }

    if let Some(name) = current_name.take() {
        sections
            .entry(name)
            .or_insert_with(Vec::new)
            .push(current_content.join("\n").trim().to_string());
    }

    sections
}
