use crate::mode;
use crate::AutomationPhase;
use rand::{self, Rng};
use std::sync::atomic::{AtomicBool, Ordering};

static CPU_CONTROLS_ACTIVE: AtomicBool = AtomicBool::new(false);

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

    if !CPU_CONTROLS_ACTIVE.swap(true, Ordering::Relaxed) {
        println!("[auto-unlock-chars] CPU controls activated");
    }

    // Don't press jump on every frame
    let mut rng = rand::thread_rng();
    let should_jump = rng.gen_bool(0.1);

    (*controller_data).buttons = if should_jump { 0x4 } else { 0 };
    (*controller_data).stick_x = 1.0;
    (*controller_data).stick_y = 0.0;
    (*controller_data).clamped_lstick_x = 1.0;
    (*controller_data).clamped_lstick_y = 0.0;
    (*controller_data).clamped_rstick_x = 0.0;
    (*controller_data).clamped_rstick_y = 0.0;
}
