use std::sync::{Arc, RwLock};

use crate::debugger::debugger::Debugger;
use crate::debugger::video_driver::VideoDriver;
use crate::gbr::game_boy::GameBoy;

pub struct DebuggerApp {}

impl DebuggerApp {
    pub fn run(game_boy: Arc<RwLock<GameBoy>>) {
        Self::init_logger();

        let debugger = Debugger::attach(game_boy.clone());

        log::info!("creating video");
        let driver = VideoDriver::new(800, 600);

        driver
            .start(debugger)
            .expect("Failed to start video driver");
    }

    fn init_logger() {
        dotenv::dotenv().ok();
        pretty_env_logger::try_init().ok();
    }
}
