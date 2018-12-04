#[derive(Debug, Deserialize, Serialize)]
pub struct MapConfig {
    pub size: (u32, u32),
    pub spawn1: (f32, f32),
    pub spawn2: (f32, f32),
    pub map: Vec<i32>,
}
impl Default for MapConfig {
    fn default() -> Self {
        MapConfig {
            size: (20, 15),
            spawn1: (36.0, 36.0),
            spawn2: (588.0, 428.0),
            map: vec![],
        }
    }
}
