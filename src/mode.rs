use crate::hooks::OFFSET_DRAW;
use crate::AutomationPhase;
use skyline;
use skyline::nn::ro::LookupSymbol;
use skyline::nn::ui2d::Layout;
use smash::app::lua_bind::FighterManager as FighterManagerBindings;
use smash::app::FighterManager as FighterManagerObject;

use std::sync::{OnceLock, RwLock};

static CURRENT_PHASE: RwLock<AutomationPhase> = RwLock::new(AutomationPhase::Other);
static FIGHTER_MANAGER_ADDR: OnceLock<usize> = OnceLock::new();

#[skyline::hook(offset = OFFSET_DRAW)]
pub unsafe fn hook_draw(layout: *mut Layout, draw_info: u64, cmd_buffer: u64) {
    let layout_name = unsafe { skyline::from_c_str((*layout).layout_name) };
    let phase = match layout_name.as_str() {
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
    original!()(layout, draw_info, cmd_buffer)
}

pub fn install_hooks() {
    lookup_fighter_manager_addr();
    skyline::install_hook!(hook_draw);
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
        .unwrap_or_default()
}
