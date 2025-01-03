use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x10203478;

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x9C0)]
pub struct UWorld {
    #[field(offset = 0x150)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x160)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,

    #[field(offset = 0x9B4)]
    pub position: [f32; 3],
}

#[raw_struct(size = 0xC0)]
pub struct ULevel {
    #[field(offset = 0xB8)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0x68)]
pub struct GameInstance {
    /*#[field(offset = 0x60)]
    pub local_players: Ptr64<dyn EncryptedTArray<dyn ULocalPlayer>>,*/
    #[field(offset = 0x60)]
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

#[raw_struct(size = 0x398)]
pub struct AActor {
    #[field(offset = 0x10)]
    pub id: u32,

    #[field(offset = 0x390)]
    pub root_component: EncryptedPtr64<dyn USceneComponent>,
}

#[raw_struct(size = 0x4E8)]
pub struct APlayerController {
    #[field(offset = 0x440)]
    pub player_state: Ptr64<()>,

    #[field(offset = 0x4B8)]
    pub acknowledged_pawn: Ptr64<dyn APawn>,

    #[field(offset = 0x4E0)]
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

#[raw_struct(size = 0x1744)]
pub struct ACharacter {
    #[field(offset = 0x308)]
    pub health_flag: u8,

    #[field(offset = 0x1740)]
    pub team: u32,

    #[field(offset = 0x4F0)]
    pub mesh: Ptr64<u64>,

    #[field(offset = 0xA48)]
    pub health: f32,

    #[field(offset = 0xA40)]
    pub health1: u32,

    #[field(offset = 0xA30)]
    pub health2: f32,

    #[field(offset = 0xA5C)]
    pub health3: u8,

    #[field(offset = 0xA5D)]
    pub health5: u8,

    #[field(offset = 0xA58)]
    pub health6: u32,
}
impl APawn for dyn ACharacter {}

#[raw_struct(size = 0x1CF4)]
pub struct APlayerCameraManager {
    #[field(offset = 0x175C)]
    pub camera_rot: [f32; 3],

    #[field(offset = 0x1768)]
    pub camera_pos: [f32; 3],

    #[field(offset = 0x1CF0)]
    pub camera_fov: f32,
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0x1FC)]
pub struct USceneComponent {
    #[field(offset = 0x1F0)]
    pub relative_location: [f32; 3],
}
