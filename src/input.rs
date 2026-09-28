use anyhow::Result;
use skyline::nn::hid::NpadHandheldState;

#[allow(improper_ctypes)]
extern "C" {
    pub fn add_nn_hid_hook(callback: fn(*mut NpadHandheldState, *const u32));
}

pub fn get_npad_state_start(_state: *mut NpadHandheldState, _controller_id: *const u32) {}

pub fn get_cursor_position() -> Option<(u32, u32)> {
    // Returns none if cursor is not available
    None
}

pub fn move_cursor(_x: u32, _y: u32) -> Result<()> {
    // Returns err if the cursor is not available
    Ok(())
}

pub fn press_a() {}

pub fn press_start() {}
