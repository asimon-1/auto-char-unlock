use crate::mode;
use crate::AutomationPhase;
use anyhow::Result;
use skyline::nn::hid::NpadHandheldState;

static STAGE_SELECT_TARGET_COORDS: (f32, f32) = (430.0, 400.0); // Final Destination
static CHAR_SELECT_TARGET_COORDS: (f32, f32) = (-860.0, 415.0); // Mario
static CURSOR_POS_THRESHOLD: f32 = 5.0;

#[allow(improper_ctypes)]
extern "C" {
    pub fn add_nn_hid_hook(callback: fn(*mut NpadHandheldState, *const u32));
}

pub fn get_npad_state_start(_state: *mut NpadHandheldState, _controller_id: *const u32) {}

pub fn get_cursor_position() -> Option<(f32, f32)> {
    match mode::get_current_phase() {
        AutomationPhase::CharacterSelect | AutomationPhase::StageSelect => mode::CURSOR_POS
            .try_read()
            .map(|phase| *phase)
            .inspect_err(|e| {
                println!(
                    "[auto-unlock-chars] Cound not acquire read lock to CURSOR_POS: {:?}",
                    e
                );
            })
            .ok(),
        _ => None,
    }
}

pub fn move_cursor(_x: u32, _y: u32) -> Result<()> {
    // Returns err if the cursor is not available
    Ok(())
}

pub fn press_a() {}

pub fn press_start() {}
