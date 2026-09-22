use std::time::Duration;

use smithay_client_toolkit::reexports::calloop::{
    EventLoop,
    timer::{TimeoutAction, Timer},
};

use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;

use smithay_client_toolkit::reexports::client::{
    Connection,
    globals::registry_queue_init,
};

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell,
        },
        WaylandSurface,
    },
    shm::Shm,
    shm::slot::SlotPool,
};

use slint::{
    ComponentHandle, PhysicalSize,
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};

mod state;
mod platform;
mod rendering;
mod input;
mod surface;
mod clock;

use crate::wayland::platform::SierraPlatform;
use crate::wayland::state::SierraState;

const WIDTH: u32 = 300;
const HEIGHT: u32 = 60;

pub fn run() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");

    let (globals, event_queue) =
        registry_queue_init(&conn).expect("Failed to initialize Wayland registry");

    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");

    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell is not available");

    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let surface = compositor.create_surface(&qh);

    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("sierra_island"), None);

    layer.set_anchor(Anchor::TOP);
    layer.set_size(WIDTH, HEIGHT);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_exclusive_zone(0);
    layer.set_margin(2, 0, 0, 0);
    layer.commit();

    let pool =
        SlotPool::new((WIDTH * HEIGHT * 4) as usize, &shm).expect("Failed to create wl_shm pool");

    let slint_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);

    slint::platform::set_platform(Box::new(SierraPlatform {
        window: slint_window.clone(),
    }))
    .expect("Failed to install Slint platform");

    slint_window.set_size(PhysicalSize::new(WIDTH, HEIGHT));

    let island = crate::Island::new().expect("Failed to create Slint Island");

    let registry = crate::containers::ContainerRegistry::default_layout();

    let container_model = crate::containers::slint_model::container_registry_to_model(&registry);

    island
        .global::<crate::ContainerState>()
        .set_containers(container_model);

    let theme = crate::theme::Theme::load();

    crate::themer::apply_theme(&island, &theme);

    crate::wayland::clock::update_time_state(&island);

    island.show().expect("Failed to show Slint Island");

    island.window().request_redraw();

    let mut event_loop: EventLoop<SierraState> =
        EventLoop::try_new().expect("Failed to initialize calloop event loop");

    let loop_signal = event_loop.get_signal();

    let mut state = SierraState {
        registry_state: RegistryState::new(&globals),

        seat_state: SeatState::new(&globals, &qh),

        output_state: OutputState::new(&globals, &qh),

        shm,

        layer,

        pool,

        buffer: None,

        slint_window,

        island,

        pointer: None,

        loop_signal: loop_signal.clone(),

        width: WIDTH,

        height: HEIGHT,

        configured: false,

        exit: false,
    };

    let loop_handle = event_loop.handle();

    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .expect("Failed to insert Wayland source");

    loop_handle
        .insert_source(
            Timer::from_duration(Duration::from_millis(16)),
            |_, _, state| {
                slint::platform::update_timers_and_animations();

                state.draw();

                TimeoutAction::ToDuration(Duration::from_millis(16))
            },
        )
        .expect("Failed to insert Slint timer");

    loop_handle
        .insert_source(
            Timer::from_duration(Duration::from_secs(1)),
            |_, _, state| {
                state.update_time();

                TimeoutAction::ToDuration(Duration::from_secs(1))
            },
        )
        .expect("Failed to insert clock timer");

    let (theme_sender, theme_channel) =
        smithay_client_toolkit::reexports::calloop::channel::channel::<()>();

    crate::themer::start_watcher(theme_sender);

    loop_handle
        .insert_source(theme_channel, |event, _, state| {
            if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(()) = event {
                let theme = crate::theme::Theme::load();

                crate::themer::apply_theme(&state.island, &theme);

                state.island.window().request_redraw();

                println!("[UI] Theme applied.");
            }
        })
        .expect("Failed to insert theme watcher");

    event_loop
        .run(None, &mut state, |_| {})
        .expect("Calloop event loop failed");
}
