#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BoundedU32<const MIN: u32, const MAX: u32>(u32);

impl<const MIN: u32, const MAX: u32> BoundedU32<MIN, MAX> {
    pub fn new(value: u32) -> Option<Self> {
        if value >= MIN && value <= MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}
