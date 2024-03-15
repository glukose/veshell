use std::env;

use smithay::output::Output;
use smithay::reexports::calloop::{channel, EventSource};
use smithay::{
    backend::allocator::dmabuf::Dmabuf,
    reexports::wayland_server::{
        backend::{ClientData, ClientId, DisconnectReason},
        protocol::wl_surface::{self},
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            with_surface_tree_downward, CompositorClientState, CompositorHandler,
            SurfaceAttributes, TraversalAction,
        },
        dmabuf::DmabufHandler,
        shell::xdg::XdgShellHandler,
        shm::ShmHandler,
    },
};

use crate::flutter_engine::platform_channels::binary_messenger::BinaryMessenger;
use crate::flutter_engine::FlutterEngine;
use crate::mouse_button_tracker::MouseButtonTracker;
use crate::server::ServerState;

mod cursor;
mod drm_backend;
mod flutter_engine;
mod gles_framebuffer_importer;
mod input_handling;
mod keyboard;
mod mouse_button_tracker;
mod server;
mod texture_swap_chain;
mod x11_client;
mod focus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    if env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok() {
        x11_client::run_x11_client();
    } else {
        drm_backend::run_drm_backend();
    }

    Ok(())
}

pub trait Backend {
    fn seat_name(&self) -> String;

    fn get_monitor_layout(&self) -> Vec<Output>;
}

pub struct FlutterState<BackendData: Backend + 'static> {
    pub flutter_engine: FlutterEngine<BackendData>,
    pub mouse_button_tracker: MouseButtonTracker,
}

pub struct CalloopData<BackendData: Backend + 'static> {
    pub state: ServerState<BackendData>,
    pub tx_fbo: channel::Sender<Option<Dmabuf>>,
    pub batons: Vec<flutter_engine::Baton>,
}

pub fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been commited
            for callback in states
                .cached_state
                .current::<SurfaceAttributes>()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("disconnected");
    }
}
