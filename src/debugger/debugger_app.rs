use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::thread;

use rustyline::Editor;

use crate::debugger::commands::*;
use crate::debugger::debugger::Debugger;
use crate::debugger::video_driver::VideoDriver;
use crate::gbr::game_boy::GameBoy;

pub struct DebuggerApp {
    debugger: Arc<Debugger>,
}

impl DebuggerApp {
    pub fn new(game_boy: Arc<RwLock<GameBoy>>) -> Self {
        DebuggerApp {
            debugger: Arc::new(Debugger::new(game_boy.clone())),
        }
    }

    pub fn run(&mut self) {
        self.init_logger();

        let debugger = self.debugger.clone();
        thread::spawn(|| {
            DebuggerApp::run_prompt(debugger);
        });

        log::info!("creating video");
        let driver = VideoDriver::new(self.debugger.clone(), 800, 600);

        driver.start().expect("Failed to start video driver");
    }

    fn run_prompt(debugger: Arc<Debugger>) {
        let mut rl = Editor::<()>::new();
        let mut stop_sig: Option<Sender<()>> = None;
        loop {
            match rl.readline("gb-r> ") {
                Ok(line) => match line.parse() {
                    Ok(Command::RunStop) => {
                        log::info!("run/stop");
                        match stop_sig {
                            Some(stop) => {
                                stop.send(()).unwrap();
                                stop_sig = None;
                            }
                            None => stop_sig = Some(debugger.run()),
                        }
                    }
                    Ok(Command::Step) => debugger.step(),
                    Ok(Command::Quit) => {
                        if let Some(stop) = stop_sig {
                            stop.send(()).unwrap();
                        }
                        break;
                    }
                    Err(_) => println!("Unknown command"),
                },
                Err(e) => {
                    log::warn!("prompt error: {}", e);
                    break;
                }
            }
        }
    }

    fn init_logger(&self) {
        dotenv::dotenv().ok();
        pretty_env_logger::try_init().ok();
    }
}
