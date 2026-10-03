mod fighter_control;
mod hooks;
mod input;
mod mode;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AutomationPhase {
    StageSelect,
    CharacterSelect,
    MatchPlaying,
    ResultsScreen,
    NewFighterAppeared,
    NewFighterResult,
    Milestone,
    Other,
    #[default]
    Disabled,
}

#[skyline::main(name = "auto-unlock-chars")]
pub fn main() {
    println!("[auto-unlock-chars] Initializing plugin");
    mode::init();
    input::initialize_stick_action_count();
    hooks::install_hooks();
    println!("[auto-unlock-chars] Hooks installed");
}
