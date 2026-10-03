mod fighter_control;
mod hooks;
mod input;
mod mode;

#[skyline::main(name = "auto-unlock-chars")]
pub fn main() {
    println!("[auto-unlock-chars] Initializing plugin");
    hooks::disable_rumble();
    mode::init();
    hooks::install_hooks();
    println!("[auto-unlock-chars] Hooks installed");
}
