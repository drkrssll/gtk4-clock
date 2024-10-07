use std::{
    env,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

use chrono::Local;
use gio::glib::{clone::Downgrade, timeout_add_local};
use gtk4::{
    gdk::Display,
    glib::{timeout_add_seconds_local, ExitCode},
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

fn detect_wayland() -> bool {
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or_default();

    session_type.contains("wayland")
        || (!wayland_display.is_empty() && !session_type.contains("x11"))
}

fn event_listener(window_sender: Sender<bool>) {
    let instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("HYPRLAND_INSTANCE_SIGNATURE not found. Is Hyprland running?");

    let socket_path = format!("/run/user/1000/hypr/{}/.socket2.sock", instance_signature);

    thread::spawn(move || {
        if let Ok(mut stream) = UnixStream::connect(&socket_path) {
            let _ = stream.write_all(b"subscribewindow\n");

            let reader = BufReader::new(stream);
            for line in reader.lines() {
                if let Ok(event) = line {
                    if event.contains("fullscreen>>1") {
                        let _ = window_sender.send(true);
                    } else if event.contains("fullscreen>>0") {
                        let _ = window_sender.send(false);
                    }
                }
            }
        }
    });
}

fn main() -> ExitCode {
    let application = Application::builder()
        .application_id("clock.widget")
        .build();

    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(app: &Application) {
    let clock = Label::new(None);
    clock.set_widget_name("clock_label");

    let window = ApplicationWindow::builder()
        .application(app)
        .title("GTK4 Clock")
        .child(&clock)
        .build();

    window.set_decorated(false);
    window.set_resizable(false);

    if detect_wayland() {
        setup_wayland_window(&window);
    }

    load_css();
    handle_time(&clock);

    window.present();
}

fn setup_wayland_window(window: &ApplicationWindow) {
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

    let (window_sender, window_receiver) = channel::<bool>();
    event_listener(window_sender);

    let window_weak = window.downgrade();
    timeout_add_local(Duration::from_millis(100), move || {
        if let Ok(is_fullscreen) = window_receiver.try_recv() {
            if let Some(window) = window_weak.upgrade() {
                if is_fullscreen {
                    window.hide();
                } else {
                    window.show();
                }
            }
        }
        gio::glib::ControlFlow::Continue
    });
}

fn handle_time(clock_label: &Label) {
    let clock_clone = clock_label.clone();

    let current_time = Local::now();
    let initial_text = format!(
        "<span background='#000000' foreground='#FFFFFF' size='large'>{}</span>\n<span foreground='#fabbc2' weight='bold' size='small'>{}</span><span foreground='#FF0110' weight='bold' size='small'>{}</span>",
        current_time.format("%A").to_string(),
        current_time.format("%b ").to_string(),
        current_time.format("%d").to_string(),
    );

    clock_clone.set_markup(&initial_text);

    timeout_add_seconds_local(1, move || {
        let clock_clone = clock_clone.clone();
        timeout_add_seconds_local(1, move || {
            let current_time = Local::now();

            let formatted_time = format!(
                "<span foreground='#FFFFFF' size='large'>{}</span>\n<span foreground='#FFFFFF' size='large'>  {}</span> <span foreground='#FF0110' weight='bold' size='small'>{}</span>",
                current_time.format("%I").to_string(),
                current_time.format("%M").to_string(),
                current_time.format("%p").to_string()
            );

            clock_clone.set_markup(&formatted_time);

            gio::glib::ControlFlow::Continue
        });

        gio::glib::ControlFlow::Break
    });
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(STYLE);

    if let Some(display) = Display::default() {
        style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
