use crate::{ObjectRef, TransformTRS};

#[derive(Debug, Clone)]
pub struct MeshInstance {
    pub name: String,
    pub target: ObjectRef,
    pub transform: TransformTRS,
    pub source_file: Option<String>,
}
