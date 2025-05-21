use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x1061C468;

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x950)]
pub struct UWorld {
    #[field(offset = 0x948)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x30)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,
}

#[raw_struct(size = 0x158)]
pub struct ULevel {
    #[field(offset = 0x150)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0xF0)]
pub struct GameInstance {
    /*#[field(offset = 0x60)]
    pub local_players: Ptr64<dyn EncryptedTArray<dyn ULocalPlayer>>,*/
    #[field(offset = 0xE8)]
    pub local_player: Ptr64<EncryptedPtr64<dyn ULocalPlayer>>,
}

#[raw_struct(size = 0x38)]
pub struct ULocalPlayer {
    #[field(offset = 0x30)]
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

#[raw_struct(size = 0x170)]
pub struct AActor {
    #[field(offset = 0x18)]
    pub id: u32,

    #[field(offset = 0x168)]
    pub root_component: EncryptedPtr64<dyn USceneComponent>,
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

#[raw_struct(size = 0x26D4)]
pub struct APawn {
    #[field(offset = 0x1880)]
    pub last_team_num: u32,

    #[field(offset = 0x26D0)]
    pub spectated_count: u32,
}
impl AActor for dyn APawn {}

#[raw_struct(size = 0xA46)]
pub struct ACharacter {
    #[field(offset = 0x2D8)]
    pub health_flag: u8,

    #[field(offset = 0x6E0)]
    pub mesh: Ptr64<u64>,

    #[field(offset = 0xA40)]
    pub health: f32,

    #[field(offset = 0x968)]
    pub health1: u32,

    #[field(offset = 0x990)]
    pub health2: f32,

    #[field(offset = 0xA54)]
    pub health3: u8,

    #[field(offset = 0xA55)]
    pub health5: u8,

    #[field(offset = 0xA50)]
    pub health6: u32,
}
impl APawn for dyn ACharacter {}

#[raw_struct(size = 0x1664)]
pub struct APlayerCameraManager {
    #[field(offset = 0x1658)]
    pub camera_rot: [f32; 3],

    #[field(offset = 0x1648)]
    pub camera_pos: [f32; 3],
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0x2CC)]
pub struct USceneComponent {
    #[field(offset = 0x2C0)]
    pub relative_location: [f32; 3],
}
