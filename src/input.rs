use crate::mode;
use crate::mode::AutomationPhase;
use skyline::nn::hid::NpadHandheldState;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static STAGE_SELECT_TARGET_COORDS: (f32, f32) = (200.0, 400.0); // Small Battlefield
static CHAR_SELECT_TARGET_COORDS: (f32, f32) = (0.0, 180.0);
static CURSOR_POS_THRESHOLD: f32 = 15.0;
static BUTTON_HOLD_FRAMES: AtomicU8 = AtomicU8::new(0);
static CURSOR_HOLD_FRAMES: AtomicU8 = AtomicU8::new(0);
static STICK_INPUT_FRAMES: AtomicU8 = AtomicU8::new(0);
pub static CHARACTER_SELECTED: AtomicBool = AtomicBool::new(false);

const KEY_RIGHT: u64 = 0x4000;
const KEY_LEFT: u64 = 0x1000;
const KEY_DOWN: u64 = 0x8000;
const KEY_UP: u64 = 0x2000;
const KEY_A: u64 = 0x1;
const KEY_START: u64 = 0x400;
const MAX_BUTTON_HOLD_FRAMES: u8 = 12;
const MAX_CURSOR_HOLD_FRAMES: u8 = 64;

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

pub fn move_cursor(state: *mut NpadHandheldState) {
    if state.is_null() {
        return;
    }

    let phase = mode::get_current_phase();
    let cursor_target = match phase {
        AutomationPhase::StageSelect => STAGE_SELECT_TARGET_COORDS,
        AutomationPhase::CharacterSelect => CHAR_SELECT_TARGET_COORDS,
        _ => return,
    };
    let cursor_position = get_cursor_position().ok_or_else(|| return).unwrap();
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
        let held_frames = CURSOR_HOLD_FRAMES.load(Ordering::Relaxed);
        if held_frames >= MAX_CURSOR_HOLD_FRAMES {
            CURSOR_HOLD_FRAMES.store(0, Ordering::Relaxed);
            return;
        }
        CURSOR_HOLD_FRAMES.store(held_frames + 1, Ordering::Relaxed);
        unsafe {
            (*state).Buttons |= cursor_buttons;
        }
    } else {
        CURSOR_HOLD_FRAMES.store(0, Ordering::Relaxed);
    }
}

pub fn reset_character_selected() {
    CHARACTER_SELECTED.store(false, Ordering::Relaxed);
}

pub fn press_buttons(state: *mut NpadHandheldState) {
    if state.is_null() {
        return;
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
                if CHARACTER_SELECTED.load(Ordering::Relaxed) {
                    KEY_START
                } else {
                    CHARACTER_SELECTED.store(true, Ordering::Relaxed);
                    KEY_A
                }
            } else {
                0
            }
        }
        AutomationPhase::MatchPlaying => 0,
        _ => KEY_A,
    };

    if buttons != 0 {
        let held_frames = BUTTON_HOLD_FRAMES.load(Ordering::Relaxed);
        if held_frames >= MAX_BUTTON_HOLD_FRAMES {
            BUTTON_HOLD_FRAMES.store(0, Ordering::Relaxed);
            return;
        }
        BUTTON_HOLD_FRAMES.store(held_frames + 1, Ordering::Relaxed);

        unsafe {
            (*state).Buttons |= buttons;
        }
    } else {
        BUTTON_HOLD_FRAMES.store(0, Ordering::Relaxed);
    }
}

pub fn move_stick(state: *mut NpadHandheldState) {
    if state.is_null() {
        return;
    }

    let phase = mode::get_current_phase();
    if phase != AutomationPhase::MatchPlaying {
        STICK_INPUT_FRAMES.store(0, Ordering::Relaxed);
        return;
    }

    let input_frame = STICK_INPUT_FRAMES.fetch_add(1, Ordering::Relaxed) % 4;
    if input_frame >= 2 {
        unsafe {
            (*state).LStickX = 0;
            (*state).LStickY = 0;
        }
        return;
    }

    unsafe {
        (*state).LStickX = 0;
        (*state).LStickY = i32::MIN;
    }
}
