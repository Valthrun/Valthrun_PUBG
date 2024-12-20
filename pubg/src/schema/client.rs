use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x10263AF8;

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x168)]
pub struct UWorld {
    #[field(offset = 0x150)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x160)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,
}

#[raw_struct(size = 0x50)]
pub struct ULevel {
    #[field(offset = 0x48)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0xE8)]
pub struct GameInstance {
    /*#[field(offset = 0xE0)]
    pub local_players: Ptr64<dyn EncryptedTArray<dyn ULocalPlayer>>,*/
    #[field(offset = 0xE0)]
    pub local_player: Ptr64<EncryptedPtr64<dyn ULocalPlayer>>,
}

#[raw_struct(size = 0x38)]
pub struct ULocalPlayer {
    #[field(offset = 0x30)]
    pub player_controller: EncryptedPtr64<dyn APlayerController>,
}

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

#[raw_struct(size = 0xA46)]
pub struct AActor {
    #[field(offset = 0x280)]
    pub root_component: EncryptedPtr64<dyn USceneComponent>,

    #[field(offset = 0x380)]
    pub health_flag: u8,

    #[field(offset = 0x4C8)]
    pub mesh: Ptr64<u64>,

    #[field(offset = 0xA30)]
    pub health: f32,

    #[field(offset = 0xA28)]
    pub health1: u32,

    #[field(offset = 0x970)]
    pub health2: f32,

    #[field(offset = 0xA44)]
    pub health3: u8,

    #[field(offset = 0xA45)]
    pub health5: u8,

    #[field(offset = 0xA40)]
    pub health6: u32,
}

#[raw_struct(size = 0x4F0)]
pub struct APlayerController {
    #[field(offset = 0x438)]
    pub player_state: Ptr64<()>,

    #[field(offset = 0x4C0)]
    pub acknowledged_pawn: Ptr64<dyn APawn>,

    #[field(offset = 0x4E8)]
    pub player_camera_manager: Ptr64<dyn APlayerCameraManager>,
}
impl AActor for dyn APlayerController {}

#[raw_struct(size = 0x126C)]
pub struct APawn {
    #[field(offset = 0xEA0)]
    pub last_team_num: u32,

    #[field(offset = 0x1268)]
    pub spectated_count: u32,
}
impl AActor for dyn APawn {}

#[raw_struct(size = 0x1720)]
pub struct APlayerCameraManager {
    #[field(offset = 0x1708)]
    pub camera_rot: [f32; 3],

    #[field(offset = 0x1714)]
    pub camera_pos: [f32; 3],
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0x21C)]
pub struct USceneComponent {
    #[field(offset = 0x210)]
    pub relative_location: [f32; 3],
}
