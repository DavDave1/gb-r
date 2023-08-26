use std::sync::{Arc, RwLock};

use crate::debugger::debugger::Debugger;
use crate::debugger::video_driver::VideoDriver;
use crate::gbr::game_boy::GameBoy;

pub struct DebuggerApp {}

impl DebuggerApp {
    pub fn run(game_boy: Arc<RwLock<GameBoy>>) {
        Self::init_logger();

        // thread::spawn(|| {
        //     DebuggerApp::run_prompt(debugger);
        // });

        let debugger = Debugger::attach(game_boy.clone());

        log::info!("creating video");
        let driver = VideoDriver::new(800, 600);

        driver
            .start(debugger)
            .expect("Failed to start video driver");
    }

    // fn run_prompt(debugger: Arc<Debugger>) {
    //     let mut rl = DefaultEditor::new().unwrap();
    //     let mut cmd_sig: Option<Sender<DebuggerCommand>> = None;
    //     loop {
    //         match rl.readline("gb-r> ") {
    //             Ok(line) => match line.parse() {
    //                 Ok(Command::RunStop) => {
    //                     log::info!("run/stop");
    //                     match cmd_sig.as_ref() {
    //                         Some(sig) => {
    //                             sig.send(DebuggerCommand::Stop).unwrap();
    //                             cmd_sig = None;
    //                         }
    //                         None => cmd_sig = Some(debugger.run()),
    //                     }
    //                 }
    //                 Ok(Command::Step) => debugger.step(),
    //                 Ok(Command::Quit) => {
    //                     if let Some(sig) = cmd_sig {
    //                         sig.send(DebuggerCommand::Stop).unwrap();
    //                     }
    //                     break;
    //                 }
    //                 Err(_) => println!("Unknown command"),
    //             },
    //             Err(e) => {
    //                 log::warn!("prompt error: {}", e);
    //                 break;
    //             }
    //         }
    //     }
    // }

    fn init_logger() {
        dotenv::dotenv().ok();
        pretty_env_logger::try_init().ok();
    }
}
