use crate::fighter_control;
use crate::input;
use crate::mode;
use nnsdk::ui2d::Layout;
use skyline::nn::hid::{GetNpadHandheldState, NpadHandheldState};

fn hook_panic() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        let message = if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            *message
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            message.as_str()
        } else {
            "Box<Any>"
        };

        let error_message = format!("thread has panicked at '{}', {}", message, location);
        skyline::error::show_error(
            69,
            "auto-char-unlock plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            error_message.as_str(),
        );
    }));
}

#[skyline::hook(offset = 0x4b620)] // 13.0.5
pub unsafe fn hook_draw(layout: *mut Layout, draw_info: u64, cmd_buffer: u64) {
    let layout_name = unsafe { skyline::from_c_str((*layout).layout_name) };
    mode::update_phase(&layout_name);
    if let Some(root_pane) = unsafe { (*layout).root_pane.as_ref() } {
        mode::update_cursor_pos(root_pane);
    }
    original!()(layout, draw_info, cmd_buffer)
}

#[skyline::hook(replace = GetNpadHandheldState)]
pub unsafe fn hook_get_npad_handheld_state(
    state: *mut NpadHandheldState,
    controller_id: *const u32,
) {
    original!()(state, controller_id);
    input::move_cursor(state);
    input::press_buttons(state);
    input::move_stick(state);
}

#[skyline::hook(offset = 0x2da180)] // 13.0.5
pub unsafe fn hook_set_cpu_controls(control_data: *mut *mut u8) {
    original!()(control_data);
    fighter_control::set_cpu_controls_selfdestruct(control_data);
}

pub fn install_hooks() {
    hook_panic();
    skyline::install_hook!(hook_draw);
    skyline::install_hook!(hook_get_npad_handheld_state);
    skyline::install_hook!(hook_set_cpu_controls);
}
