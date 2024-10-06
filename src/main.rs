use std::{
    cell::RefCell,
    env,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    rc::Rc,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use chrono::Local;
use gtk4::{
    gdk::Display,
    glib::{idle_add_local, timeout_add_seconds_local, ExitCode},
    prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt},
    style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, Label,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

const STYLE: &str = "
window {
    background-color: transparent;
}

#clock_label {
    font-size: 34px;
    font-family: feather;
    font-family: Iosevka;
    background-color: #000000;
    color: #FFFFFF;
    padding: 10px;
    border: 2px solid black;
    border-radius: 20px;
}
";

enum WLDisplay {}

const SOCKET_PATH: &str = "/run/user/1000/hypr/0f594732b063a90d44df8c5d402d658f27471dfe_1728240250_31247147/.socket2.sock";

impl WLDisplay {
    fn detect_wayland() -> bool {
        let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
        let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();

        session_type.contains("wayland")
            || (!wayland_display.is_empty() && !session_type.contains("x11"))
    }

    fn event_listener(window_sender: mpsc::Sender<bool>) {
        thread::spawn(move || {
            if let Ok(mut stream) = UnixStream::connect(SOCKET_PATH) {
                let _ = stream.write_all(b"subscribewindow\n");

                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    if let Ok(event) = line {
                        println!("Received event: {}", event);

                        if event.contains("fullscreen>>1") {
                            let _ = window_sender.send(true);
                        } else if event.contains("fullscreen>>0") {
                            let _ = window_sender.send(false);
                        }

                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });
    }
}

fn main() -> ExitCode {
    let _ = gtk4::init();

    load_css();

    let app = Application::builder()
        .application_id("clock.widget")
        .build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let clock = Label::new(None);
    clock.set_widget_name("clock_label");

    handle_time(&clock);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("GTK4 Clock")
        .default_width(180)
        .default_height(75)
        .build();

    window.set_decorated(false);
    window.set_resizable(false);

    window.set_child(Some(&clock));

    if WLDisplay::detect_wayland() {
        window.init_layer_shell();

        window.set_layer(Layer::Overlay);
        window.set_margin(Edge::Right, 20);
        window.set_margin(Edge::Top, 20);

        let anchors = [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ];

        for (anchor, state) in anchors {
            window.set_anchor(anchor, state);
        }

        window.present();

        let (window_sender, window_receiver) = mpsc::channel::<bool>();

        WLDisplay::event_listener(window_sender);

        let window = Rc::new(RefCell::new(window.clone()));

        idle_add_local(move || {
            if let Ok(is_fullscreen) = window_receiver.try_recv() {
                let window = window.borrow();
                if !is_fullscreen {
                    window.show()
                } else {
                    window.hide()
                }
            }
            gtk4::glib::ControlFlow::Continue
        });
    }
}

fn handle_time(clock_label: &Label) {
    let current_time = Local::now();

    let formatted_time = format!(
        "<span background='#000000' foreground='#FFFFFF' size='large'>{}</span> <span foreground='#FF0110' weight='bold' size='small'>{}</span>",
        current_time.format("%I:%M").to_string(),
        current_time.format("%p").to_string()
    );

    clock_label.set_markup(&formatted_time);

    let clock_label_clone = clock_label.clone();
    timeout_add_seconds_local(1, move || {
        let current_time = Local::now();

        let formatted_time = format!(
            "<span foreground='#FFFFFF' size='large'>{}</span> <span foreground='#FF0110' weight='bold' size='small'>{}</span>",
            current_time.format("%I:%M").to_string(),
            current_time.format("%p").to_string()
        );

        clock_label_clone.set_markup(&formatted_time);

        gtk4::glib::ControlFlow::Continue
    });
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(STYLE);

    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    )
}
