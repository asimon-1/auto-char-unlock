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
    Other,
    #[default]
    Disabled,
}

#[skyline::main(name = "auto-unlock-chars")]
pub fn main() {
    mode::init();
    hooks::install_hooks();
}
