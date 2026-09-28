use crate::mode;
use crate::AutomationPhase;
use anyhow::Result;
use skyline::nn::hid::NpadHandheldState;
use std::sync::atomic::{AtomicU8, Ordering};

static STAGE_SELECT_TARGET_COORDS: (f32, f32) = (430.0, 400.0); // Final Destination
static CHAR_SELECT_TARGET_COORDS: (f32, f32) = (-860.0, 415.0); // Mario
static CURSOR_POS_THRESHOLD: f32 = 15.0;
static BUTTON_HOLD_FRAMES: AtomicU8 = AtomicU8::new(0);

const KEY_RIGHT: u64 = 0x4000;
const KEY_LEFT: u64 = 0x1000;
const KEY_DOWN: u64 = 0x8000;
const KEY_UP: u64 = 0x2000;
const KEY_A: u64 = 0x1;
const KEY_START: u64 = 0x400;
const MAX_BUTTON_HOLD_FRAMES: u8 = 8;

#[allow(improper_ctypes)]
extern "C" {
    pub fn add_nn_hid_hook(callback: fn(*mut NpadHandheldState, *const u32));
}

pub fn get_npad_state_start(state: *mut NpadHandheldState, _controller_id: *const u32) {
    let _ = move_cursor(state);
    let _ = press_buttons(state);
}

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

pub fn move_cursor(state: *mut NpadHandheldState) -> Result<()> {
    if state.is_null() {
        anyhow::bail!("controller state is unavailable");
    }

    let phase = mode::get_current_phase();
    let cursor_target = match phase {
        AutomationPhase::StageSelect => STAGE_SELECT_TARGET_COORDS,
        AutomationPhase::CharacterSelect => CHAR_SELECT_TARGET_COORDS,
        _ => anyhow::bail!("cursor movement is unavailable in the current phase"),
    };
    let cursor_position =
        get_cursor_position().ok_or_else(|| anyhow::anyhow!("cursor position is unavailable"))?;
    let mut cursor_buttons = 0;
    if cursor_position.0 < cursor_target.0 - CURSOR_POS_THRESHOLD {
        cursor_buttons |= KEY_RIGHT;
    } else if cursor_position.0 > cursor_target.0 + CURSOR_POS_THRESHOLD {
        cursor_buttons |= KEY_LEFT;
    }
    if cursor_position.1 < cursor_target.1 - CURSOR_POS_THRESHOLD {
        cursor_buttons |= KEY_UP;
    } else if cursor_position.1 > cursor_target.1 + CURSOR_POS_THRESHOLD {
        cursor_buttons |= KEY_DOWN;
    }

    if cursor_buttons != 0 {
        unsafe {
            (*state).Buttons |= cursor_buttons;
        }
    }
    Ok(())
}

pub fn press_buttons(state: *mut NpadHandheldState) -> Result<()> {
    if state.is_null() {
        anyhow::bail!("controller state is unavailable");
    }

    let phase = mode::get_current_phase();
    let target = match phase {
        AutomationPhase::CharacterSelect => Some(CHAR_SELECT_TARGET_COORDS),
        AutomationPhase::StageSelect => Some(STAGE_SELECT_TARGET_COORDS),
        _ => None,
    };
    let close_to_target = if let Some(target) = target {
        if let Some(cursor_position) = get_cursor_position() {
            (cursor_position.0 - target.0).abs() < CURSOR_POS_THRESHOLD
                && (cursor_position.1 - target.1).abs() < CURSOR_POS_THRESHOLD
        } else {
            false
        }
    } else {
        false
    };

    let buttons = match phase {
        AutomationPhase::StageSelect => {
            if close_to_target {
                KEY_A
            } else {
                0
            }
        }
        AutomationPhase::CharacterSelect => {
            if close_to_target {
                KEY_A
            } else {
                0
            }
        }
        AutomationPhase::NewFighterAppeared
        | AutomationPhase::NewFighterResult
        | AutomationPhase::ResultsScreen => KEY_A,
        _ => 0,
    };

    if buttons != 0 {
        let held_frames = BUTTON_HOLD_FRAMES.load(Ordering::Relaxed);
        if held_frames < MAX_BUTTON_HOLD_FRAMES {
            BUTTON_HOLD_FRAMES.store(held_frames + 1, Ordering::Relaxed);
        } else {
            return Ok(());
        }

        unsafe {
            (*state).Buttons |= buttons;
        }
    } else {
        BUTTON_HOLD_FRAMES.store(0, Ordering::Relaxed);
    }
    Ok(())
}
