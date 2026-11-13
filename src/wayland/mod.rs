use smithay_client_toolkit::reexports::calloop::{EventLoop, channel::channel};

use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;

use smithay_client_toolkit::reexports::client::{Connection, globals::registry_queue_init};

use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell, shm::Shm, shm::slot::SlotPool,
};

use slint::{
    ComponentHandle, PhysicalSize,
    platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};

use std::time::Instant;

mod animation;
mod handlers;
mod input;
mod keyboard;
mod output_surface;
mod platform;
mod profiling;
mod rendering;
pub mod state;
mod surface;
mod time;

use crate::wayland::{
    platform::SierraPlatform,
    state::{EventCommand, SierraState},
    surface::{COLLAPSED_HEIGHT, COLLAPSED_WIDTH, HEIGHT, WIDTH},
};

pub fn run() {
    let mut startup_prof = crate::wayland::profiling::Stats::new();

    let conn_timer = crate::wayland::profiling::Timer::start();
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    startup_prof.record_startup("wayland_connect", conn_timer.elapsed_ns());

    let (globals, event_queue) =
        registry_queue_init(&conn).expect("Failed to initialize Wayland registry");

    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");

    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell is not available");

    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let output_state = OutputState::new(&globals, &qh);

    let pool =
        SlotPool::new((WIDTH * HEIGHT * 4) as usize, &shm).expect("Failed to create wl_shm pool");

    let slint_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);

    let platform_timer = crate::wayland::profiling::Timer::start();
    slint::platform::set_platform(Box::new(SierraPlatform {
        window: slint_window.clone(),
    }))
    .expect("Failed to install Slint platform");
    startup_prof.record_startup("slint_platform", platform_timer.elapsed_ns());

    slint_window.set_size(PhysicalSize::new(COLLAPSED_WIDTH, COLLAPSED_HEIGHT));

    let island_timer = crate::wayland::profiling::Timer::start();
    let island = crate::Island::new().expect("Failed to create Slint Island");
    startup_prof.record_startup("island_creation", island_timer.elapsed_ns());

    let registry = crate::containers::ContainerRegistry::default_layout();

    let container_model = crate::containers::slint_model::container_registry_to_model(&registry);

    island
        .global::<crate::ContainerState>()
        .set_containers(container_model);

    let theme_timer = crate::wayland::profiling::Timer::start();
    let theme = crate::theme::Theme::load();
    startup_prof.record_startup("theme_load", theme_timer.elapsed_ns());

    crate::themer::apply_theme(&island, &theme);

    crate::wayland::time::update_time_state(&island);

    // Initialize and retain application discovery in memory
    let discovery = std::rc::Rc::new(std::cell::RefCell::new(
        crate::discovery::ApplicationDiscovery::default(),
    ));
    let (search_tx, search_rx) =
        std::sync::mpsc::channel::<(Vec<slint::SharedString>, Vec<String>)>();
    let discovery_timer = crate::wayland::profiling::Timer::start();
    if let Ok(()) = discovery.borrow_mut().discover_applications() {
        startup_prof.record_startup("app_discovery", discovery_timer.elapsed_ns());
        // Set up a global callback for input changes
        let callbacks = island.global::<crate::InputCallbacks>();
        let discovery_clone = discovery.clone();

        callbacks.on_on_input_change(move |query: slint::SharedString| {
            let discovery = discovery_clone.borrow();
            let matches = discovery.search_applications(query.as_str());
            let (names, ids): (Vec<_>, Vec<_>) = matches
                .iter()
                .map(|app| {
                    let display = app.display_name.clone();
                    let truncated = if display.chars().count() > 30 {
                        let t: String = display.chars().take(27).collect();
                        format!("{}…", t)
                    } else {
                        display
                    };
                    (slint::SharedString::from(truncated), app.name.clone())
                })
                .unzip();
            let _ = search_tx.send((names, ids));
        });
    }

    island.set_visible_requested(false);

    island.show().expect("Failed to show Slint Island");

    island.window().request_redraw();

    let event_loop_init_timer = crate::wayland::profiling::Timer::start();
    let mut event_loop: EventLoop<SierraState> =
        EventLoop::try_new().expect("Failed to initialize calloop event loop");
    startup_prof.record_startup("event_loop_init", event_loop_init_timer.elapsed_ns());

    let (cmd_sender, cmd_channel) = channel::<EventCommand>();

    let loop_handle = event_loop.handle();

    let mut state = SierraState {
        registry_state: RegistryState::new(&globals),

        compositor_state: compositor,
        layer_shell,
        qh: qh.clone(),
        loop_handle: loop_handle.clone(),
        surfaces: Vec::new(),
        seat_state: SeatState::new(&globals, &qh),
        output_state,
        shm,
        pool,
        slint_buffer: None,

        slint_window,

        island,

        pointer: None,
        keyboard: None,
        island_keyboard_entered: false,
        width: COLLAPSED_WIDTH,

        height: COLLAPSED_HEIGHT,

        configured: false,

        trigger_hovered: false,

        island_hovered: false,

        island_expanded: false,

        island_visible: false,

        refresh_rate_mhz: 60000,

        command_sender: Some(cmd_sender),

        close_timer_pending: false,

        collapse_timer_pending: false,

        close_timer_token: None,

        collapse_timer_token: None,

        search_results: slint::ModelRc::from(Vec::<slint::SharedString>::new().as_slice()),
        search_results_rx: search_rx,
        search_results_full: Vec::new(),
        discovery: discovery.clone(),

        height_target: COLLAPSED_HEIGHT,
        height_anim_from: COLLAPSED_HEIGHT,
        height_anim_start: Instant::now(),
        height_animating: false,
        launch_dismissed: false,
        prof: startup_prof,
    };

    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .expect("Failed to insert Wayland source");

    let cmd_lh = loop_handle.clone();
    loop_handle
        .insert_source(cmd_channel, move |event, _, state| {
            if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(
                EventCommand::PointerEnter,
            ) = event
            {
                crate::wayland::input::handle_pointer_enter(state, &cmd_lh);
            }

            if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(
                EventCommand::PointerLeave,
            ) = event
            {
                crate::wayland::input::handle_pointer_leave(state, &cmd_lh);
            }

            if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(
                EventCommand::ScheduleCollapseTimer,
            ) = event
            {
                crate::wayland::input::schedule_collapse_timer(state, &cmd_lh);
            }
        })
        .expect("Failed to insert command channel");

    crate::wayland::time::install_clock_timer(&loop_handle);

    crate::themer::install_watcher(&loop_handle);

    crate::wayland::animation::install_animation_timer(&loop_handle);

    event_loop
        .run(None, &mut state, |_| {})
        .expect("Calloop event loop failed");
}
