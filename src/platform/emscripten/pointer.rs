extern crate stdweb;

use super::Window2;
use crate::dpi::LogicalPosition;
use crate::dpi::LogicalSize;
use crate::events::ModifiersState;
use crate::events::Pointer;
use crate::events::PointerButton;
use crate::events::PointerPhase;
use crate::events::PointerType;

use std::sync::Arc;

use self::stdweb::traits::*;
use self::stdweb::unstable::TryInto;
use self::stdweb::web::event::PointerDownEvent;
use self::stdweb::web::event::PointerMoveEvent;
use self::stdweb::web::event::PointerUpEvent;
use self::stdweb::web::html_element::CanvasElement;
use self::stdweb::*;

/// Register web Pointer Events on the emscripten canvas to send winit Pointer events.
///
/// Note: We may get double events for mouse and touch (via this and the existing mouse/touch pointer events)?
pub fn register(window: Arc<Window2>) {
    let canvas: CanvasElement = js! {
        return window.Module.canvas;
    }
    .try_into()
    .unwrap();

    canvas.add_event_listener({
        let window = window.clone();
        move |event: PointerDownEvent| {
            window
                .events
                .lock()
                .unwrap()
                .push_back(::Event::WindowEvent {
                    window_id: ::WindowId(unsafe { super::WindowId::dummy() }),
                    event: ::WindowEvent::Pointer(translate_pointer_event(
                        PointerPhase::Down,
                        &event,
                    )),
                });
        }
    });

    canvas.add_event_listener({
        let window = window.clone();
        move |event: PointerMoveEvent| {
            // todo get_coalesced_events not supported everywhere?
            for pointer_event in event.get_coalesced_events() {
                window
                    .events
                    .lock()
                    .unwrap()
                    .push_back(::Event::WindowEvent {
                        window_id: ::WindowId(unsafe { super::WindowId::dummy() }),
                        event: ::WindowEvent::Pointer(translate_pointer_event(
                            PointerPhase::Move,
                            &pointer_event,
                        )),
                    });
            }
        }
    });

    canvas.add_event_listener({
        let window = window.clone();
        move |event: PointerUpEvent| {
            window
                .events
                .lock()
                .unwrap()
                .push_back(::Event::WindowEvent {
                    window_id: ::WindowId(unsafe { super::WindowId::dummy() }),
                    event: ::WindowEvent::Pointer(translate_pointer_event(
                        PointerPhase::Up,
                        &event,
                    )),
                });

            // TODO get coalesed?
        }
    });
}

fn translate_pointer_event(phase: PointerPhase, event: &impl IPointerEvent) -> Pointer {
    use self::stdweb::web::event::MouseButton;

    Pointer {
        id: event.pointer_id() as u32,
        position: LogicalPosition::new(event.offset_x(), event.offset_y()),
        modifiers: ModifiersState {
            shift: event.shift_key(),
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            logo: event.meta_key(),
        },
        phase,
        is_primary: event.is_primary(),
        pointer_type: match event.pointer_type().as_str() {
            "pen" => PointerType::Pen,
            "mouse" => PointerType::Mouse,
            "touch" => PointerType::Touch,
            _ => unreachable!(),
        },
        size: LogicalSize::new(event.width(), event.height()),
        pressure: event.pressure(),
        tangential_pressure: event.tangential_pressure(),
        tilt_x: event.tilt_x() as f64 / 90.0,
        tilt_y: event.tilt_y() as f64 / 90.0,
        twist: event.twist() as f64 / 360.0 * (2.0 * std::f64::consts::PI),

        // TODO we're not dealing with buttons here?
        button: PointerButton::None, // button: match event.button() {
                                     //     MouseButton::Left => PointerButton::Left,
                                     //     MouseButton::Wheel => PointerButton::Middle,
                                     //     MouseButton::Right => PointerButton::Right,
                                     //     MouseButton::Button4 => PointerButton::Other(4),
                                     //     MouseButton::Button5 => PointerButton::Other(5),
                                     // },
    }
}
