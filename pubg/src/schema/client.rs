use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x10CE0C98;

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x0118)]
pub struct UWorld {
    #[field(offset = 0x00E0)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x0110)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,
}

#[raw_struct(size = 0x0068)]
pub struct ULevel {
    #[field(offset = 0x0060)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0x0060)]
pub struct GameInstance {
    /*#[field(offset = 0x60)]
    pub local_players: Ptr64<dyn EncryptedTArray<dyn ULocalPlayer>>,*/
    #[field(offset = 0x0058)]
    pub local_player: Ptr64<EncryptedPtr64<dyn ULocalPlayer>>,
}

#[raw_struct(size = 0x0040)]
pub struct ULocalPlayer {
    #[field(offset = 0x0038)]
    pub player_controller: EncryptedPtr64<dyn APlayerController>,
}
impl AActor for dyn ULocalPlayer {}

#[raw_struct(size = 0x10)]
pub struct TArray<T>
where
    T: Send + Sync + 'static,
{
    #[field(offset = 0x0)]
    pub data: Ptr64<[T]>,

    #[field(offset = 0x8)]
    pub count: u32,

    #[field(offset = 0x10)]
    pub max: u32,
}

#[raw_struct(size = 0x10)]
pub struct EncryptedTArray<T>
where
    T: Send + Sync + 'static,
{
    #[field(offset = 0x0)]
    pub data: EncryptedPtr64<[T]>,

    #[field(offset = 0x8)]
    pub count: u32,

    #[field(offset = 0xC)]
    pub max: u32,
}

#[raw_struct(size = 0x0108)]
pub struct AActor {
    #[field(offset = 0x0018)]
    pub id: u32,

    #[field(offset = 0x0100)]
    pub root_component: EncryptedPtr64<dyn USceneComponent>,
}

#[raw_struct(size = 0x04D8)]
pub struct APlayerController {
    #[field(offset = 0x04D0)]
    pub player_camera_manager: Ptr64<dyn APlayerCameraManager>,
}
impl AActor for dyn APlayerController {}

#[raw_struct(size = 0x173C)]
pub struct APawn {
    #[field(offset = 0x1738)]
    pub last_team_num: u32,
}
impl AActor for dyn APawn {}

#[raw_struct(size = 0x0A56)]
pub struct ACharacter {
    #[field(offset = 0x0270)]
    pub health_flag: u8,

    #[field(offset = 0x0A40)]
    pub health: f32,

    #[field(offset = 0x0990)]
    pub health1: u32,

    #[field(offset = 0x09A0)]
    pub health2: f32,

    #[field(offset = 0x0A54)]
    pub health3: u8,

    #[field(offset = 0x0A55)]
    pub health5: u8,

    #[field(offset = 0x0A50)]
    pub health6: u32,
}
impl APawn for dyn ACharacter {}

#[raw_struct(size = 0x1C04)]
pub struct APlayerCameraManager {
    #[field(offset = 0x1658)]
    pub camera_rot: [f32; 3],

    #[field(offset = 0x1BF8)]
    pub camera_pos: [f32; 3],
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0x035C)]
pub struct USceneComponent {
    #[field(offset = 0x0350)]
    pub relative_location: [f32; 3],
}
