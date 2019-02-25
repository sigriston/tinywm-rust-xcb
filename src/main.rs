use xcb;
use std::cmp::max;

struct ButtonPress {
    id: xcb::Window,
    root_x: i16,
    root_y: i16,
}

impl ButtonPress {
    fn from_event(event: xcb::GenericEvent) -> Option<Self> {
        let button_press: &xcb::ButtonPressEvent = unsafe {
            xcb::cast_event(&event)
        };
        let id = button_press.child();

        if id == xcb::NONE {
            None
        } else {
            Some(ButtonPress {
                id,
                root_x: button_press.root_x(),
                root_y: button_press.root_y(),
            })
        }
    }
}

struct WindowGeom {
    id: xcb::Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl WindowGeom {
    fn from_window_id(conn: &xcb::Connection, id: xcb::Window) -> Self {
        let cookie = xcb::get_geometry(&conn, id);
        let geom = cookie.get_reply().unwrap();
        WindowGeom {
            id,
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

    let setup = conn.get_setup();
    let screen = setup.roots().nth(default_screen_num as usize).unwrap();
    let default_root_window = screen.root();

    // TODO: Not possible now
    // xcb::grab_key();

    // grab Alt + Button1
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

    // grab Alt + Button3
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

    // STATE
    let mut button_press: Option<ButtonPress> = None;
    let mut start: Option<WindowGeom> = None;

    loop {
        let event = conn.wait_for_event();
        match event {
            None => { break; }
            Some(event) => {
                let r = event.response_type() & !0x80;
                // println!("EVENT!");
                match r {
                    xcb::BUTTON_PRESS => {
                        let bpe = ButtonPress::from_event(event);

                        // we only care about the button press if it initiated inside a child
                        // window
                        if bpe.is_some() {
                            button_press = bpe;
                            let button_press = button_press.as_ref().unwrap();

                            start = Some(WindowGeom::from_window_id(
                                &conn,
                                button_press.id,
                            ));
                        }
                    }
                    xcb::MOTION_NOTIFY => {
                        let mne: &xcb::MotionNotifyEvent  = unsafe {
                            xcb::cast_event(&event)
                        };

                        if let (&Some(ref button_press), &Some(ref window_geom)) = (&button_press, &start) {
                            let root_x = mne.root_x();
                            let root_y = mne.root_y();

                            let state = mne.state();

                            let xdiff = root_x - button_press.root_x;
                            let ydiff = root_y - button_press.root_y;

                            let final_x = window_geom.x + match state & xcb::KEY_BUT_MASK_BUTTON_1 as u16 {
                                0 => 0,
                                _ => xdiff,
                            };
                            let final_y = window_geom.y + match state & xcb::KEY_BUT_MASK_BUTTON_1 as u16 {
                                0 => 0,
                                _ => ydiff,
                            };
                            let final_width = max(1i16, window_geom.width as i16 + match state & xcb::KEY_BUT_MASK_BUTTON_3 as u16 {
                                0 => 0,
                                _ => xdiff,
                            });
                            let final_height = max(1i16, window_geom.height as i16 + match state & xcb::KEY_BUT_MASK_BUTTON_3 as u16 {
                                0 => 0,
                                _ => ydiff,
                            });

                            xcb::configure_window(
                                &conn,
                                window_geom.id,
                                &[
                                    (xcb::CONFIG_WINDOW_X as u16, final_x as u32),
                                    (xcb::CONFIG_WINDOW_Y as u16, final_y as u32),
                                    (xcb::CONFIG_WINDOW_WIDTH as u16, final_width as u32),
                                    (xcb::CONFIG_WINDOW_HEIGHT as u16, final_height as u32),
                                ],
                            );
                            conn.flush();
                        }
                    }
                    xcb::BUTTON_RELEASE => {
                        button_press = None;
                        start = None;
                    }
                    _ => {}
                }
            }
        }
    }
}
