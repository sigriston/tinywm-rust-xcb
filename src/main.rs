use xcb;
use std::cmp::max;

/// Tracks the start of a mouse drag
struct DragStart {
    /// x coordinate of button press that started the drag
    root_x: i16,
    /// y coordinate of button press that started the drag
    root_y: i16,
    /// child window that's the target of the drag
    child: xcb::Window,
    /// geometry of child window
    child_geom: WindowGeom,
}

impl DragStart {
    /// Creates DragStart from a xcb event (must be a button press)
    fn from_event(conn: &xcb::Connection, event: xcb::GenericEvent) -> Option<Self> {
        let button_press: &xcb::ButtonPressEvent = unsafe {
            xcb::cast_event(&event)
        };
        let child = button_press.child();

        // if button press is not on a window, we don't care
        if child == xcb::NONE {
            None
        } else {
            Some(Self {
                root_x: button_press.root_x(),
                root_y: button_press.root_y(),
                child,
                child_geom: WindowGeom::from_window_id(conn, child),
            })
        }
    }
}

/// Basic window geometry structure
struct WindowGeom {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl WindowGeom {
    /// Creates a WindowGeom structure given the window's id
    fn from_window_id(conn: &xcb::Connection, id: xcb::Window) -> Self {
        let cookie = xcb::get_geometry(&conn, id);
        let geom = cookie.get_reply().unwrap();
        WindowGeom {
            x: geom.x(),
            y: geom.y(),
            width: geom.width(),
            height: geom.height(),
        }
    }
}

fn main() {
    // `xcb::Connection::connect(None)` will connect to the default display, or whatever display is
    // set on the `$DISPLAY` environment variable.
    // if it returns an error, unwrap will pick it up and panic, and the program will terminate.
    let (conn, default_screen_num) = xcb::Connection::connect(None).unwrap();

    // obtain id of the root window; only works when there is a single screen, as we only get the
    // root of the default screen
    let setup = conn.get_setup();
    let screen = setup.roots().nth(default_screen_num as usize).unwrap();
    let default_root_window = screen.root();

    // grab Alt + Button1 (moving windows)
    xcb::grab_button(
        &conn,
        true,
        default_root_window,
        (xcb::EVENT_MASK_BUTTON_PRESS | xcb::EVENT_MASK_BUTTON_RELEASE | xcb::EVENT_MASK_POINTER_MOTION) as u16,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_ASYNC as u8,
        default_root_window,
        xcb::NONE,
        xcb::BUTTON_INDEX_1 as u8,
        xcb::MOD_MASK_1 as u16,
    );

    // grab Alt + Button3 (resizing windows)
    xcb::grab_button(
        &conn,
        true,
        default_root_window,
        (xcb::EVENT_MASK_BUTTON_PRESS | xcb::EVENT_MASK_BUTTON_RELEASE | xcb::EVENT_MASK_POINTER_MOTION) as u16,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_ASYNC as u8,
        default_root_window,
        xcb::NONE,
        xcb::BUTTON_INDEX_3 as u8,
        xcb::MOD_MASK_1 as u16,
    );

    // flush to ensure grab requests are honored
    conn.flush();

    // used to save info of when a mouse drag starts
    let mut drag_start: Option<DragStart> = None;

    loop {
        // synchronously get the next event, similar to XNextEvent() from Xlib
        let event = conn.wait_for_event();
        match event {
            // None is returned in case of I/O error; we'll just bail in that case
            None => { break; }
            Some(event) => {
                // the response type identifies the kind of event we're getting here, and it's a
                // sequential u8; the most significant bit is supposed to be masked out
                let r = event.response_type() & !0x80;
                match r {
                    xcb::BUTTON_PRESS => {
                        // in case of a button press we want to remember it as the start of a mouse
                        // drag; this will be None in case we don't care (when it's not targeting a
                        // client window)
                        drag_start = DragStart::from_event(&conn, event);
                    }
                    xcb::MOTION_NOTIFY => {
                        let mne: &xcb::MotionNotifyEvent  = unsafe {
                            xcb::cast_event(&event)
                        };

                        // only do anything if this is a true window drag
                        if let Some(ref drag_start) = drag_start {
                            let root_x = mne.root_x();
                            let root_y = mne.root_y();

                            // state is a flags field indicating which buttons are being held
                            let state = mne.state();

                            // calculate how far we've dragged from the start
                            let xdiff = root_x - drag_start.root_x;
                            let ydiff = root_y - drag_start.root_y;

                            // if Button1 is being held, we're moving the window, so modify its x/y
                            let final_x = drag_start.child_geom.x + match state & xcb::KEY_BUT_MASK_BUTTON_1 as u16 {
                                0 => 0,
                                _ => xdiff,
                            };
                            let final_y = drag_start.child_geom.y + match state & xcb::KEY_BUT_MASK_BUTTON_1 as u16 {
                                0 => 0,
                                _ => ydiff,
                            };

                            // if Button3 is being held, we're resizing, in which case we'll modify
                            // the window's width and height; the max function ensures that width
                            // and height will be at least 1, preventing overflows or, worse, 0s
                            // which are fatal
                            let final_width = max(1i16, drag_start.child_geom.width as i16 + match state & xcb::KEY_BUT_MASK_BUTTON_3 as u16 {
                                0 => 0,
                                _ => xdiff,
                            });
                            let final_height = max(1i16, drag_start.child_geom.height as i16 + match state & xcb::KEY_BUT_MASK_BUTTON_3 as u16 {
                                0 => 0,
                                _ => ydiff,
                            });

                            // reconfigure the window with the computed position and dimensions
                            xcb::configure_window(
                                &conn,
                                drag_start.child,
                                &[
                                    (xcb::CONFIG_WINDOW_X as u16, final_x as u32),
                                    (xcb::CONFIG_WINDOW_Y as u16, final_y as u32),
                                    (xcb::CONFIG_WINDOW_WIDTH as u16, final_width as u32),
                                    (xcb::CONFIG_WINDOW_HEIGHT as u16, final_height as u32),
                                ],
                            );

                            // we flush every time... might not be the best for perf but is easy
                            conn.flush();
                        }
                    }
                    xcb::BUTTON_RELEASE => {
                        // when the mouse button is release we're no longer dragging
                        drag_start = None;
                    }
                    _ => {}
                }
            }
        }
    }
}
