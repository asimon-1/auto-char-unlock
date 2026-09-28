//mod cpu_control;
mod hooks;
mod input;
//mod menu;
mod mode;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AutomationPhase {
    StageSelect,
    CharacterSelect,
    MatchPlaying,
    ResultsScreen,
    NewFighterAppeared,
    NewFighterResult,
    Other,
    #[default]
    Disabled,
}

// fn main_loop() {
// while true,
// read mode::CURRENT_PHASE
// match current phase
//     StageSelect => Move cursor to preset then press A
//     CharacterSelect => Move cursor to preset then press A then press start
//     MatchLoading => No action
//     MatchPlaying => if !is_entry_level_9 then enable_level_9_for_entry
//     ResultsScreen => Press Start
//     NewFighterAppeared => Press A
//     NewFighterMatchPlaying => if !is_entry_level_9 then enable_level_9_for_entry
//     NewFighterResult => Press A
//     Other => No action
// }

#[skyline::main(name = "auto-unlock-chars")]
pub fn main() {
    println!("[auto-unlock-chars] hello");
    unsafe {
        mode::init();
        hooks::install_hooks();
    }
    // spawn new thread for main_loop
}
