use chrono::Local;
use gtk4::{
    gdk::Display,
    glib::{timeout_add_seconds_local, ExitCode},
    prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt},
    style_context_add_provider_for_display, Application, ApplicationWindow, CssProvider, Label,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

const STYLE: &str = "
#clock_label {
    font-size: 42px;
    font-family: feather;
    font-family: Iosevka;
    background-color: #000000;
    color: #FFFFFF;
    padding: 10px;
    border: 2px solid black;
}
";

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
    let clock_label = Label::new(None);
    clock_label.set_widget_name("clock_label");

    handle_time(&clock_label);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("GTK4 Clock")
        .default_width(230)
        .default_height(90)
        .build();

    window.set_child(Some(&clock_label));
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

    window.set_decorated(false);
    window.set_resizable(false);

    window.present()
}

fn handle_time(clock_label: &Label) {
    let current_time = Local::now();

    let formatted_time = format!(
        "<span foreground='#FFFFFF' size='large'>{}</span> <span foreground='#FF0110' weight='bold' size='small'>{}</span>",
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

        glib::ControlFlow::Continue
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
