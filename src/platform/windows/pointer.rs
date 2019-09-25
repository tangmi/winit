use crate::events::Pointer;
use crate::events::PointerButton;
use crate::events::PointerPhase;
use crate::events::PointerType;
use LogicalPosition;
use LogicalSize;

use winapi::shared::windef::HWND;
use winapi::shared::windef::POINT;
use winapi::um::winuser;

unsafe fn pixel_location_to_pointer_position(
    hwnd: HWND,
    pixel_location: POINT,
    dpi_factor: f64,
) -> LogicalPosition {
    // TODO: this could be calculated from the himetric if we have the size of the current screen in himetric and pixels
    //       This requires `winuser::GetPointerDeviceRects`, which is currently missing in winapi

    let mut position = pixel_location;
    let res = winuser::ScreenToClient(hwnd, &mut position);
    assert!(res != 0);
    LogicalPosition::from_physical((position.x as f64, position.y as f64), dpi_factor)
}

pub fn get_pointer_coalesced_events(
    pointer_id: u32,
    modifiers: crate::events::ModifiersState,
    dpi_factor: f64,
    phase: PointerPhase,
) -> Vec<Pointer> {
    unsafe {
        let mut pointer_type: winuser::POINTER_INPUT_TYPE = 0;
        winuser::GetPointerType(pointer_id, &mut pointer_type);

        match pointer_type {
            winuser::PT_POINTER => {
                // Generic pointer type. This type never appears in pointer messages
                unreachable!()
            }

            winuser::PT_TOUCH => {
                let touch_info_history = {
                    let mut entries_count = 0;
                    winuser::GetPointerTouchInfoHistory(
                        pointer_id,
                        &mut entries_count,
                        std::ptr::null_mut(),
                    );
                    let mut touch_info_history: Vec<winuser::POINTER_TOUCH_INFO> =
                        Vec::with_capacity(entries_count as usize);
                    winuser::GetPointerTouchInfoHistory(
                        pointer_id,
                        &mut entries_count,
                        touch_info_history.as_mut_ptr(),
                    );
                    touch_info_history.set_len(entries_count as usize);
                    touch_info_history
                };

                touch_info_history
                    .iter()
                    .map(|touch_info| {
                        let size = if (touch_info.touchMask & winuser::TOUCH_MASK_CONTACTAREA) != 0
                        {
                            LogicalSize::new(
                                (touch_info.rcContact.right - touch_info.rcContact.left).abs()
                                    as f64,
                                (touch_info.rcContact.top - touch_info.rcContact.bottom).abs()
                                    as f64,
                            )
                        } else {
                            LogicalSize::new(0.0, 0.0)
                        };

                        let pressure = if (touch_info.touchMask & winuser::TOUCH_MASK_PRESSURE) != 0
                        {
                            touch_info.pressure as f64 / 1024.0
                        } else {
                            0.0
                        };

                        let twist = if (touch_info.touchMask & winuser::TOUCH_MASK_ORIENTATION) != 0
                        {
                            touch_info.orientation as f64 / 360.0 * (2.0 * std::f64::consts::PI)
                        } else {
                            0.0
                        };

                        Pointer {
                            id: pointer_id,
                            position: pixel_location_to_pointer_position(
                                touch_info.pointerInfo.hwndTarget,
                                touch_info.pointerInfo.ptPixelLocation,
                                dpi_factor,
                            ),
                            modifiers,
                            phase,
                            is_primary: false, // todo
                            pointer_type: PointerType::Touch,
                            size,
                            pressure,
                            tangential_pressure: 0.0,
                            tilt_x: 0.0,
                            tilt_y: 0.0,
                            twist,
                            button: PointerButton::None, // todo?
                        }
                    })
                    .collect()
            }

            winuser::PT_PEN => {
                let pen_info_history = {
                    let mut entries_count = 0;
                    winuser::GetPointerPenInfoHistory(
                        pointer_id,
                        &mut entries_count,
                        std::ptr::null_mut(),
                    );
                    let mut pen_info_history: Vec<winuser::POINTER_PEN_INFO> =
                        Vec::with_capacity(entries_count as usize);
                    winuser::GetPointerPenInfoHistory(
                        pointer_id,
                        &mut entries_count,
                        pen_info_history.as_mut_ptr(),
                    );
                    pen_info_history.set_len(entries_count as usize);
                    pen_info_history
                };

                pen_info_history
                    .iter()
                    .map(|pen_info| {
                        if (pen_info.penFlags & winuser::PEN_FLAG_BARREL) != 0 {
                            // barrel button is pressed
                        }

                        if (pen_info.penFlags & winuser::PEN_FLAG_INVERTED) != 0 {
                            // the pen is inverted (probably for the second type of pen, e.g. brush instead of pencil)
                        }

                        if (pen_info.penFlags & winuser::PEN_FLAG_ERASER) != 0 {
                            // eraser button is pressed
                        }

                        let pressure = if (pen_info.penMask & winuser::PEN_MASK_PRESSURE) != 0 {
                            pen_info.pressure as f64 / 1024.0
                        } else {
                            0.0
                        };

                        let tilt_x = if (pen_info.penMask & winuser::PEN_MASK_TILT_X) != 0 {
                            pen_info.tiltX as f64 / 90.0
                        } else {
                            0.0
                        };

                        let tilt_y = if (pen_info.penMask & winuser::PEN_MASK_TILT_Y) != 0 {
                            pen_info.tiltY as f64 / 90.0
                        } else {
                            0.0
                        };

                        let twist = if (pen_info.penMask & winuser::PEN_MASK_ROTATION) != 0 {
                            pen_info.rotation as f64 / 360.0 * (2.0 * std::f64::consts::PI)
                        } else {
                            0.0
                        };

                        Pointer {
                            id: pointer_id,
                            position: pixel_location_to_pointer_position(
                                pen_info.pointerInfo.hwndTarget,
                                pen_info.pointerInfo.ptPixelLocation,
                                dpi_factor,
                            ),
                            modifiers,
                            phase,
                            is_primary: false, // todo
                            pointer_type: PointerType::Pen,
                            size: LogicalSize::new(0.0, 0.0),
                            pressure,
                            tangential_pressure: 0.0,
                            tilt_x,
                            tilt_y,
                            twist,
                            button: PointerButton::None, // todo
                        }
                    })
                    .collect()
            }

            winuser::PT_MOUSE => unimplemented!(),

            winuser::PT_TOUCHPAD => unimplemented!(),

            _ => unreachable!(),
        }
    }
}
