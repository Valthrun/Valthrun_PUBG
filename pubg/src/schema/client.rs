use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x105CD7D8;

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x8B8)]
pub struct UWorld {
    #[field(offset = 0x340)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x8B0)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,
}

#[raw_struct(size = 0xF8)]
pub struct ULevel {
    #[field(offset = 0xF0)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0xD0)]
pub struct GameInstance {
    /*#[field(offset = 0x60)]
    pub local_players: Ptr64<dyn EncryptedTArray<dyn ULocalPlayer>>,*/
    #[field(offset = 0xC8)]
    pub local_player: Ptr64<EncryptedPtr64<dyn ULocalPlayer>>,
}

#[raw_struct(size = 0x40)]
pub struct ULocalPlayer {
    #[field(offset = 0x38)]
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

#[raw_struct(size = 0x358)]
pub struct AActor {
    #[field(offset = 0x24)]
    pub id: u32,

    #[field(offset = 0x350)]
    pub root_component: EncryptedPtr64<dyn USceneComponent>,
}

#[raw_struct(size = 0x4E8)]
pub struct APlayerController {
    #[field(offset = 0x430)]
    pub player_state: Ptr64<()>,

    #[field(offset = 0x4B8)]
    pub acknowledged_pawn: Ptr64<dyn APawn>,

    #[field(offset = 0x4E0)]
    pub player_camera_manager: Ptr64<dyn APlayerCameraManager>,
}
impl AActor for dyn APlayerController {}

#[raw_struct(size = 0x12CC)]
pub struct APawn {
    #[field(offset = 0x1130)]
    pub last_team_num: u32,

    #[field(offset = 0x12C8)]
    pub spectated_count: u32,
}
impl AActor for dyn APawn {}

#[raw_struct(size = 0xA28)]
pub struct ACharacter {
    #[field(offset = 0x294)]
    pub health_flag: u8,

    #[field(offset = 0x7B0)]
    pub mesh: Ptr64<u64>,

    #[field(offset = 0xA08)]
    pub health: f32,

    #[field(offset = 0xA20)]
    pub health1: u32,

    #[field(offset = 0xA24)]
    pub health2: f32,

    #[field(offset = 0xA1C)]
    pub health3: u8,

    #[field(offset = 0xA1D)]
    pub health5: u8,

    #[field(offset = 0xA18)]
    pub health6: u32,
}
impl APawn for dyn ACharacter {}

#[raw_struct(size = 0xA10)]
pub struct APlayerCameraManager {
    #[field(offset = 0x470)]
    pub camera_rot: [f32; 3],

    #[field(offset = 0xA04)]
    pub camera_pos: [f32; 3],
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0x23C)]
pub struct USceneComponent {
    #[field(offset = 0x230)]
    pub relative_location: [f32; 3],
}
