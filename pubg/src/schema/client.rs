use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x10C1DE98; // GWorld

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x01A8)]
pub struct UWorld {
    #[field(offset = 0x0050)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x1A0)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,
}

#[raw_struct(size = 0x01D8)]
pub struct ULevel {
    #[field(offset = 0x01D0)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0x00D8)]
pub struct GameInstance {
    #[field(offset = 0x00D0)]
    pub local_player: Ptr64<EncryptedPtr64<dyn ULocalPlayer>>,
}

#[raw_struct(size = 0x0040)]
pub struct ULocalPlayer {
    #[field(offset = 0x0038)]
    pub player_controller: EncryptedPtr64<dyn APlayerController>,
}
impl AActor for dyn ULocalPlayer {}

#[raw_struct(size = 0x14)]
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

#[raw_struct(size = 0x0320)]
pub struct AActor {
    #[field(offset = 0x0024)]
    pub id: u32,

    #[field(offset = 0x0318)]
    pub root_component: EncryptedPtr64<dyn USceneComponent>,
}

#[raw_struct(size = 0x04E8)]
pub struct APlayerController {
    #[field(offset = 0x04E0)]
    pub player_camera_manager: Ptr64<dyn APlayerCameraManager>,
}
impl AActor for dyn APlayerController {}

#[raw_struct(size = 0x10D4)]
pub struct APawn {
    #[field(offset = 0x10D0)]
    pub last_team_num: u32,
}
impl AActor for dyn APawn {}

#[raw_struct(size = 0x0A68)]
pub struct ACharacter {
    #[field(offset = 0x0580)]
    pub mesh: Ptr64<dyn USkeletalMeshComponent>,

    #[field(offset = 0x01C8)]
    pub health_flag: u8,

    #[field(offset = 0x0A30)]
    pub health: f32,

    #[field(offset = 0x0A50)]
    pub health1: u32,

    #[field(offset = 0x0A64)]
    pub health2: f32,

    #[field(offset = 0x0A44)]
    pub health3: u8,

    #[field(offset = 0x0A45)]
    pub health5: u8,

    #[field(offset = 0x0A40)]
    pub health6: u32,
}
impl APawn for dyn ACharacter {}

#[raw_struct(size = 0x0499)]
pub struct USkeletalMeshComponent {
    #[field(offset = 0x0498)]
    pub always_create_physics_state: u8,
}

#[raw_struct(size = 0x0AB4)]
pub struct APlayerCameraManager {
    #[field(offset = 0x0AA8)]
    pub camera_rot: [f32; 3],

    #[field(offset = 0x0A80)]
    pub camera_pos: [f32; 3],
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0x024C)]
pub struct USceneComponent {
    #[field(offset = 0x0240)]
    pub relative_location: [f32; 3],
}
