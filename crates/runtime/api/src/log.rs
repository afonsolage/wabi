use bevy_reflect::{FromReflect, Reflect};

// TODO: Change this to enum when Bevy 0.9 is out.
#[derive(Reflect, FromReflect, Debug, Default)]
pub struct LogMessage {
    pub level: u8,
    pub message: String,
}
