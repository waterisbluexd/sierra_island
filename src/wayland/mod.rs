use std::time::{Duration, Instant};

use smithay_client_toolkit::reexports::calloop::{
    EventLoop,
    channel::channel,
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
        WaylandSurface,
        wlr_layer::{
            Anchor,
            KeyboardInteractivity,
            Layer,
            LayerShell,
        },
    },
    shm::Shm,
    shm::slot::SlotPool,
};

use slint::{
    ComponentHandle,
    PhysicalSize,
    platform::software_renderer::{
        MinimalSoftwareWindow,
        RepaintBufferType,
    },
};

mod time;
mod input;
mod platform;
mod rendering;
mod state;
mod handlers;
mod surface;

use crate::wayland::{
    platform::SierraPlatform,
    state::{
        SierraState,
        EventCommand,
        CLOSE_DELAY,
        COLLAPSE_DELAY,
        DEFAULT_REFRESH_MHZ,
    },
    surface::{
        COLLAPSED_HEIGHT,
        COLLAPSED_WIDTH,
        HEIGHT,
        TRIGGER_HEIGHT,
        TRIGGER_WIDTH,
        WIDTH,
    },
};

pub fn run() {
    let conn =
        Connection::connect_to_env()
            .expect("Failed to connect to Wayland");

    let (globals, event_queue) =
        registry_queue_init(&conn)
            .expect("Failed to initialize Wayland registry");

    let qh = event_queue.handle();

    let compositor =
        CompositorState::bind(&globals, &qh)
            .expect("wl_compositor is not available");

    let layer_shell =
        LayerShell::bind(&globals, &qh)
            .expect("wlr-layer-shell is not available");

    let shm =
        Shm::bind(&globals, &qh)
            .expect("wl_shm is not available");

    let surface =
        compositor.create_surface(&qh);

    let layer =
        layer_shell.create_layer_surface(
            &qh,
            surface,
            Layer::Top,
            Some("sierra_island"),
            None,
        );

    layer.set_anchor(Anchor::TOP);
    layer.set_size(
        COLLAPSED_WIDTH,
        COLLAPSED_HEIGHT,
    );
    layer.set_keyboard_interactivity(
        KeyboardInteractivity::None,
    );
    layer.set_exclusive_zone(0);
    layer.set_margin(2, 0, 0, 0);
    layer.commit();

    let trigger_surface =
        compositor.create_surface(&qh);

    let trigger_layer =
        layer_shell.create_layer_surface(
            &qh,
            trigger_surface,
            Layer::Top,
            Some("sierra_island_trigger"),
            None,
        );

    trigger_layer.set_anchor(Anchor::TOP);
    trigger_layer.set_size(
        TRIGGER_WIDTH,
        TRIGGER_HEIGHT,
    );
    trigger_layer.set_keyboard_interactivity(
        KeyboardInteractivity::None,
    );
    trigger_layer.set_exclusive_zone(0);
    trigger_layer.set_margin(0, 0, 0, 0);
    trigger_layer.commit();

    let pool =
        SlotPool::new(
            (WIDTH * HEIGHT * 4) as usize,
            &shm,
        )
        .expect("Failed to create wl_shm pool");

    let slint_window =
        MinimalSoftwareWindow::new(
            RepaintBufferType::NewBuffer,
        );

    slint::platform::set_platform(
        Box::new(SierraPlatform {
            window: slint_window.clone(),
        }),
    )
    .expect("Failed to install Slint platform");

    slint_window.set_size(
        PhysicalSize::new(
            COLLAPSED_WIDTH,
            COLLAPSED_HEIGHT,
        ),
    );

    let island =
        crate::Island::new()
            .expect("Failed to create Slint Island");

    let registry =
        crate::containers::ContainerRegistry::default_layout();

    let container_model =
        crate::containers::slint_model::container_registry_to_model(
            &registry,
        );

    island
        .global::<crate::ContainerState>()
        .set_containers(
            container_model
        );

    let theme =
        crate::theme::Theme::load();

    crate::themer::apply_theme(
        &island,
        &theme,
    );

    crate::wayland::time::update_time_state(
        &island
    );

    island.set_visible_requested(false);

    island
        .show()
        .expect("Failed to show Slint Island");

    island.window().request_redraw();

    let mut event_loop: EventLoop<SierraState> =
        EventLoop::try_new()
            .expect("Failed to initialize calloop event loop");

    let loop_signal =
        event_loop.get_signal();

    let (cmd_sender, cmd_channel) =
        channel::<EventCommand>();

    let mut state = SierraState {
        registry_state:
            RegistryState::new(&globals),

        seat_state:
            SeatState::new(
                &globals,
                &qh,
            ),

        output_state:
            OutputState::new(
                &globals,
                &qh,
            ),

        shm,

        layer,

        trigger_layer,

        pool,

        buffer: None,

        trigger_buffer: None,

        slint_buffer: None,

        slint_window,

        island,

        pointer: None,

        loop_signal:
            loop_signal.clone(),

        width:
            COLLAPSED_WIDTH,

        height:
            COLLAPSED_HEIGHT,

        configured: false,

        trigger_configured: false,

        trigger_hovered: false,

        island_hovered: false,

        island_expanded: false,

        island_visible: false,

        exit: false,

        output_id: None,

        refresh_rate_mhz: DEFAULT_REFRESH_MHZ,

        command_sender: Some(cmd_sender),

        close_timer_pending: false,

        collapse_timer_pending: false,

        animation_ticker_running: false,

        close_timer_token: None,

        collapse_timer_token: None,

        animation_ticker_token: None,
    };

    let loop_handle =
        event_loop.handle();

    WaylandSource::new(
        conn.clone(),
        event_queue,
    )
    .insert(loop_handle.clone())
    .expect("Failed to insert Wayland source");

    let cmd_lh = loop_handle.clone();
    loop_handle
        .insert_source(
            cmd_channel,
            move |event, _, state| {
                if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(EventCommand::PointerEnter) = event {
                    state.close_timer_pending = false;
                    state.collapse_timer_pending = false;

                    if let Some(token) = state.close_timer_token.take() {
                        let _ = cmd_lh.remove(token);
                    }
                    if let Some(token) = state.collapse_timer_token.take() {
                        let _ = cmd_lh.remove(token);
                    }

                    state.show_island();
                    slint::platform::update_timers_and_animations();
                    let _ = state.draw();
                    state.ensure_ticker_running(&cmd_lh);
                }

                if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(EventCommand::PointerLeave) = event {
                    state.update_hover(Instant::now());

                    if state.close_timer_pending && state.close_timer_token.is_none() {
                        let token = cmd_lh.insert_source(
                            Timer::from_duration(CLOSE_DELAY),
                            |_, _, state| {
                                state.close_timer_pending = false;
                                state.close_timer_token = None;

                                if !state.trigger_hovered
                                    && !state.island_hovered
                                    && state.island_expanded
                                {
                                    state.hide_island();
                                    state.collapse_timer_pending = true;

                                    slint::platform::update_timers_and_animations();
                                    let _ = state.draw();

                                    let _ = state.command_sender.as_ref().map(|sender| {
                                        let _ = sender.send(EventCommand::ScheduleCollapseTimer);
                                        let _ = sender.send(EventCommand::EnsureTicker);
                                    });
                                }

                                TimeoutAction::Drop
                            },
                        );

                        if let Ok(token) = token {
                            state.close_timer_token = Some(token);
                        }
                    }

                    slint::platform::update_timers_and_animations();
                    let _ = state.draw();
                    state.ensure_ticker_running(&cmd_lh);
                }

                if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(EventCommand::EnsureTicker) = event {
                    state.ensure_ticker_running(&cmd_lh);
                }

                if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(EventCommand::ScheduleCollapseTimer) = event {
                    if state.collapse_timer_pending && state.collapse_timer_token.is_none() {
                        let token = cmd_lh.insert_source(
                            Timer::from_duration(COLLAPSE_DELAY),
                            |_, _, state| {
                                state.collapse_timer_pending = false;
                                state.collapse_timer_token = None;
                                state.collapse_island();

                                slint::platform::update_timers_and_animations();
                                let _ = state.draw();

                                let _ = state.command_sender.as_ref().map(|sender| {
                                    let _ = sender.send(EventCommand::EnsureTicker);
                                });

                                TimeoutAction::Drop
                            },
                        );

                        if let Ok(token) = token {
                            state.collapse_timer_token = Some(token);
                        }
                    }
                }
            },
        )
        .expect("Failed to insert command channel");

    let clock_lh = loop_handle.clone();
    loop_handle
        .insert_source(
            Timer::from_duration(
                Duration::from_secs(1),
            ),
            move |_, _, state| {
                state.update_time();

                slint::platform::update_timers_and_animations();
                let _ = state.draw();
                state.ensure_ticker_running(&clock_lh);

                TimeoutAction::ToDuration(
                    Duration::from_secs(1),
                )
            },
        )
        .expect("Failed to insert clock timer");

    let (theme_sender, theme_channel) =
        smithay_client_toolkit::reexports::calloop::channel::channel::<()>();

    crate::themer::start_watcher(
        theme_sender
    );

    let theme_lh = loop_handle.clone();
    loop_handle
        .insert_source(
            theme_channel,
            move |event, _, state| {
                if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(()) =
                    event
                {
                    let theme =
                        crate::theme::Theme::load();

                    crate::themer::apply_theme(
                        &state.island,
                        &theme,
                    );

                    slint::platform::update_timers_and_animations();
                    let _ = state.draw();
                    state.ensure_ticker_running(&theme_lh);
                }
            },
        )
        .expect("Failed to insert theme watcher");

    event_loop
        .run(
            None,
            &mut state,
            |_| {},
        )
        .expect("Calloop event loop failed");
}
