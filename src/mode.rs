use crate::input;
use crate::AutomationPhase;
use nnsdk::ui2d::Pane;
use skyline;
use skyline::nn::oe::ReportUserIsActive;
use skyline::nn::ro::LookupSymbol;
use smash::app::lua_bind::FighterManager as FighterManagerBindings;
use smash::app::FighterManager as FighterManagerObject;
use smash::ui2d::SmashPane;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

static CURRENT_PHASE: RwLock<AutomationPhase> = RwLock::new(AutomationPhase::Other);
pub static CURSOR_POS: RwLock<(f32, f32)> = RwLock::new((0.0, 0.0));
static FIGHTER_MANAGER_ADDR: OnceLock<usize> = OnceLock::new();
static MATCH_START_TIME: RwLock<Option<Instant>> = RwLock::new(None);
static MATCH_TIMER_READY: AtomicBool = AtomicBool::new(false);

const CPU_CONTROL_DELAY: Duration = Duration::from_secs(45);

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

pub fn check_match_timer() -> bool {
    let is_ready = MATCH_START_TIME
        .try_read()
        .map(|start_time| {
            start_time.is_some_and(|start_time| start_time.elapsed() >= CPU_CONTROL_DELAY)
        })
        .unwrap_or(false);
    if is_ready && !MATCH_TIMER_READY.swap(true, Ordering::Relaxed) {
        println!("[auto-unlock-chars] Match timer complete; CPU controls may activate");
    }
    is_ready
}

fn update_match_timer(phase: AutomationPhase) {
    if let Ok(mut start_time) = MATCH_START_TIME.try_write() {
        *start_time = (phase == AutomationPhase::MatchPlaying).then(Instant::now);
        MATCH_TIMER_READY.store(false, Ordering::Relaxed);
        if phase == AutomationPhase::MatchPlaying {
            println!("[auto-unlock-chars] Match timer started: CPU controls delay is 45 seconds");
        }
    }
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
        unsafe {
            // Prevent switch from going to sleep
            ReportUserIsActive();
        }
        match CURRENT_PHASE.try_write() {
            Ok(mut current_phase) => {
                if phase == AutomationPhase::CharacterSelect {
                    input::reset_character_selected();
                }
                update_match_timer(phase);
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
