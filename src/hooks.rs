use crate::input;
use crate::mode;

pub static OFFSET_DRAW: usize = 0x4b620; // 13.0.5

fn hook_panic() {
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        let message = if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            *message
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            message.as_str()
        } else {
            "Box<Any>"
        };

        let error_message = format!("thread has panicked at '{}', {}", message, location);
        skyline::error::show_error(
            69,
            "auto-char-unlock plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            error_message.as_str(),
        );
    }));
}

pub unsafe fn install_hooks() {
    hook_panic();
    mode::install_hooks();
    input::add_nn_hid_hook(input::get_npad_state_start);
}
