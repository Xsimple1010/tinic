use std::sync::{Arc, Mutex};

use retro_ab::core::{self, RetroContext};
use retro_ab_av::{context::RetroAvCtx, Event, EventPump, Keycode};
use retro_ab_gamepad::context::GamepadContext;

pub fn game_window_handle(
    pause_request_new_frames: &mut bool,
    event_pump: &mut EventPump,
    core_ctx: &Arc<RetroContext>,
    av_ctx: &mut RetroAvCtx,
    controller_ctx: &Arc<Mutex<GamepadContext>>,
) -> bool {
    if !*pause_request_new_frames {
        core::run(&core_ctx).unwrap();
        av_ctx.get_new_frame().expect("");
    }

    let mut need_stop = false;

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => need_stop = true,
            Event::KeyDown {
                keycode: Some(Keycode::F1),
                ..
            } => {
                //reservado para o save state
            }
            Event::KeyDown {
                keycode: Some(Keycode::F2),
                ..
            } => {
                //reservado para o save state
            }
            //pausa a rom
            Event::KeyDown {
                keycode: Some(Keycode::F3),
                ..
            } => {
                if *pause_request_new_frames {
                    if let Ok(mut controller) = controller_ctx.lock() {
                        controller.stop_thread_events();
                        *pause_request_new_frames = false
                    }
                } else {
                    if let Ok(mut controller) = controller_ctx.lock() {
                        controller.resume_thread_events();
                        *pause_request_new_frames = true
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::F5),
                ..
            } => {
                if core::reset(&core_ctx).is_err() {
                    need_stop = true;
                    break;
                }
            }
            _ => {}
        }
    }

    need_stop
}
