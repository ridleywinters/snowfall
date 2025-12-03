#[derive(serde::Serialize, serde::Deserialize)]
pub struct WindowState {
    pub position: (i32, i32),
    pub size: (u32, u32),
}
