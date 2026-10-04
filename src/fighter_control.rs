use crate::mode;
use crate::mode::AutomationPhase;
use rand::{self, Rng};
use smash::app::stage;
use smash::lib::lua_const::StageID;

const BUTTON_ATTACK: u32 = 0x1;
const BUTTON_JUMP: u32 = 0x4;

#[repr(C)]
struct ControlModuleInternal {
    vtable: *mut u8,
    controller_index: i32,
    buttons: u32,
    stick_x: f32,
    stick_y: f32,
    padding: [f32; 2],
    unknown: [u32; 8],
    clamped_lstick_x: f32,
    clamped_lstick_y: f32,
    padding2: [f32; 2],
    clamped_rstick_x: f32,
    clamped_rstick_y: f32,
}

pub unsafe fn set_cpu_controls_selfdestruct(control_data: *mut *mut u8) {
    if mode::get_current_phase() != AutomationPhase::MatchPlaying || control_data.is_null() {
        return;
    }

    let controller_data = *control_data.add(1) as *mut ControlModuleInternal;
    if controller_data.is_null() {
        return;
    }

    let mut rng = rand::thread_rng();
    let should_jump = rng.gen_bool(0.1);
    // On Snake's unlock stage, the walls prevent you from jumping off
    // Press the attack button to break the walls every once in a while
    let should_attack = (*StageID::MG_Shadowmoses..=*StageID::Battle_MG_Shadowmoses)
        .contains(&stage::get_stage_id())
        && rng.gen_bool(0.05);

    let mut buttons = 0;
    if should_attack {
        buttons |= BUTTON_ATTACK;
    };
    if should_jump {
        buttons |= BUTTON_JUMP;
    };

    (*controller_data).buttons = buttons;
    (*controller_data).stick_x = -1.0; // Run left for scrolling stages like Pac-Land
    (*controller_data).stick_y = 0.0;
    (*controller_data).clamped_lstick_x = -1.0;
    (*controller_data).clamped_lstick_y = 0.0;
    (*controller_data).clamped_rstick_x = 0.0;
    (*controller_data).clamped_rstick_y = 0.0;
}
