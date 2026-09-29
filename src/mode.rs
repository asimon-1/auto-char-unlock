use crate::input;
use crate::AutomationPhase;
use nnsdk::ui2d::Pane;
use skyline;
use skyline::nn::ro::LookupSymbol;
use smash::app::lua_bind::FighterManager as FighterManagerBindings;
use smash::app::FighterManager as FighterManagerObject;
use smash::ui2d::SmashPane;
use std::sync::{OnceLock, RwLock};

static CURRENT_PHASE: RwLock<AutomationPhase> = RwLock::new(AutomationPhase::Other);
pub static CURSOR_POS: RwLock<(f32, f32)> = RwLock::new((0.0, 0.0));
static FIGHTER_MANAGER_ADDR: OnceLock<usize> = OnceLock::new();

pub fn init() {
    lookup_fighter_manager_addr();
}

fn lookup_fighter_manager_addr() {
    unsafe {
        let mut addr: usize = 0;
        LookupSymbol(
            &mut addr,
            "_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\u{0}"
                .as_bytes()
                .as_ptr(),
        );
        let _ = FIGHTER_MANAGER_ADDR.set(addr);
    }
}

fn is_results_screen() -> bool {
    unsafe {
        let addr = *FIGHTER_MANAGER_ADDR
            .get()
            .expect("FIGHTER_MANAGER_ADDR is not initialized!");
        let mgr = *(addr as *mut *mut FighterManagerObject);
        FighterManagerBindings::is_result_mode(mgr) && FighterManagerBindings::entry_count(mgr) > 0
    }
}

pub fn get_current_phase() -> AutomationPhase {
    CURRENT_PHASE
        .try_read()
        .map(|phase| *phase)
        .inspect_err(|e| {
            println!(
                "[auto-unlock-chars] Cound not acquire read lock to CURSOR_POS: {:?}",
                e
            );
        })
        .unwrap_or_default()
}

pub fn update_phase(layout_name: &str) {
    let phase = match layout_name {
        "challenger_joined" => AutomationPhase::NewFighterResult,
        "challenger_appeared" => AutomationPhase::NewFighterAppeared,
        "info_result_window" => AutomationPhase::ResultsScreen,
        "info_melee" => {
            if is_results_screen() {
                AutomationPhase::ResultsScreen
            } else {
                AutomationPhase::MatchPlaying
            }
        }
        "stage_select2" => AutomationPhase::StageSelect,
        "chara_select" => AutomationPhase::CharacterSelect,
        _ => AutomationPhase::Other,
    };
    let current_phase = get_current_phase();
    let needs_update = (current_phase != phase) && (phase != AutomationPhase::Other);
    if needs_update {
        println!("[auto-unlock-chars] New phase detected! {:?}", phase);
        match CURRENT_PHASE.try_write() {
            Ok(mut current_phase) => {
                if phase == AutomationPhase::CharacterSelect {
                    input::reset_character_selected();
                }
                *current_phase = phase;
            }
            Err(e) => {
                println!(
                    "[auto-unlock-chars] Cound not acquire write lock to CURRENT_PHASE: {:?}",
                    e
                );
            }
        }
    }
}

pub fn update_cursor_pos(root_pane: &Pane) {
    if let Some(cursor_pane) = unsafe { root_pane.find_pane_by_name_recursive("set_hand_00") } {
        match CURSOR_POS.try_write() {
            Ok(mut cursor_pos) => {
                *cursor_pos = (cursor_pane.pos_x, cursor_pane.pos_y);
            }
            Err(e) => {
                println!(
                    "[auto-unlock-chars] Cound not acquire write lock to CURSOR_POS: {:?}",
                    e
                );
            }
        }
    }
}
