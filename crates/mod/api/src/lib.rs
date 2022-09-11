pub mod ecs;
pub mod log;
pub mod query;
pub mod registry;

#[derive(num_enum::FromPrimitive, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum Action {
    LOG,

    TEST = 254,
    #[default]
    INVALID = 255,
}
