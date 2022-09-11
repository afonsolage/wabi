use bevy_reflect::{FromReflect, Reflect};

#[derive(Reflect, FromReflect, Debug, Default)]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
}

