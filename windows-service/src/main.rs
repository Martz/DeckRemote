use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};
use tracing_subscriber;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, HC_ACTION, LLKHF_INJECTED,
};
#[cfg(windows)]
use windows::Win32::Foundation::{WPARAM, LPARAM, LRESULT};
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

// Shared state for blocked keys
type AppState = Arc<Mutex<BlockedKeysState>>;

#[derive(Clone)]
struct BlockedKeysState {
    blocked_keys: HashSet<String>,
    #[cfg(windows)]
    hook_handle: Option<HHOOK>,
}

#[derive(Serialize, Deserialize)]
struct StateResponse {
    #[serde(rename = "blockedKeys")]
    blocked_keys: Vec<String>,
}

#[derive(Deserialize)]
struct ToggleKeyRequest {
    key: String,
}

#[derive(Serialize)]
struct ToggleKeyResponse {
    success: bool,
    blocked: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    info!("DeckRemote service starting...");

    // Initialize state
    let state = Arc::new(Mutex::new(BlockedKeysState {
        blocked_keys: HashSet::new(),
        #[cfg(windows)]
        hook_handle: None,
    }));

    // Install keyboard hook on Windows
    #[cfg(windows)]
    {
        let hook_state = state.clone();
        install_keyboard_hook(hook_state);
    }

    // Setup system tray
    #[cfg(windows)]
    setup_tray();

    // Build our application with routes
    let app = Router::new()
        .route("/state", get(get_state))
        .route("/keys/toggle", post(toggle_key))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Run the server
    // NOTE: Binding to 0.0.0.0 exposes the service to all network interfaces.
    // For production use, consider:
    // - Binding to a specific interface (e.g., "192.168.1.100:7394")
    // - Using localhost only (e.g., "127.0.0.1:7394") if not needed on network
    // - Implementing authentication
    // - Using a reverse proxy with TLS
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:7394").await {
        Ok(l) => {
            info!("DeckRemote service running on http://0.0.0.0:7394");
            info!("Ready to accept connections from Stream Deck");
            info!("WARNING: Service is exposed on all network interfaces without authentication");
            l
        }
        Err(e) => {
            error!("Failed to bind to port 7394: {}", e);
            error!("Make sure no other application is using this port");
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn get_state(State(state): State<AppState>) -> Result<Json<StateResponse>, StatusCode> {
    let state = state.lock().map_err(|e| {
        error!("Failed to acquire lock: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let blocked_keys: Vec<String> = state.blocked_keys.iter().cloned().collect();
    info!("State requested - blocked keys: {:?}", blocked_keys);
    Ok(Json(StateResponse { blocked_keys }))
}

async fn toggle_key(
    State(state): State<AppState>,
    Json(payload): Json<ToggleKeyRequest>,
) -> Result<(StatusCode, Json<ToggleKeyResponse>), StatusCode> {
    let mut state = state.lock().map_err(|e| {
        error!("Failed to acquire lock: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let was_blocked = state.blocked_keys.contains(&payload.key);
    
    if was_blocked {
        state.blocked_keys.remove(&payload.key);
        info!("Key '{}' unblocked", payload.key);
    } else {
        state.blocked_keys.insert(payload.key.clone());
        info!("Key '{}' blocked", payload.key);
    }
    
    let now_blocked = !was_blocked;
    
    Ok((
        StatusCode::OK,
        Json(ToggleKeyResponse {
            success: true,
            blocked: now_blocked,
        }),
    ))
}

#[cfg(windows)]
fn install_keyboard_hook(state: AppState) {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    // SAFETY: This static is necessary because Windows hook callbacks must be static functions.
    // We use Once to ensure it's only initialized once, making it safe for concurrent access.
    // The state is Arc<Mutex<_>> which provides thread-safety for the actual data.
    static mut GLOBAL_STATE: Option<AppState> = None;
    
    unsafe {
        // Initialize the global state exactly once
        INIT.call_once(|| {
            GLOBAL_STATE = Some(state.clone());
        });
        
        let hook_handle = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            GetModuleHandleW(None).ok(),
            0,
        );
        
        if let Ok(hook) = hook_handle {
            if let Ok(mut state) = state.lock() {
                state.hook_handle = Some(hook);
                info!("Keyboard hook installed successfully");
            } else {
                error!("Failed to acquire state lock during hook installation");
            }
        } else {
            error!("Failed to install keyboard hook");
            error!("The service may need to be run with administrator privileges");
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        
        // Don't block injected events to avoid blocking legitimate input
        if kb_struct.flags & LLKHF_INJECTED == 0 {
            if let Some(ref global_state) = GLOBAL_STATE {
                // Handle potential mutex poisoning gracefully
                if let Ok(state) = global_state.lock() {
                    // Map virtual key codes to key names
                    let key_name = match kb_struct.vkCode {
                        0x5B => "LWIN",  // Left Windows key
                        0x5C => "RWIN",  // Right Windows key
                        0x5D => "APPS",  // Applications key
                        _ => "",
                    };
                    
                    if !key_name.is_empty() && state.blocked_keys.contains(key_name) {
                        // Block the key by returning non-zero
                        return LRESULT(1);
                    }
                }
                // If lock fails, don't block the key to avoid breaking keyboard input
            }
        }
    }
    
    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(windows)]
fn setup_tray() {
    std::thread::spawn(|| {
        match tray_item::TrayItem::new(
            "DeckRemote",
            tray_item::IconSource::Resource("tray-icon"),
        )
        .or_else(|_| {
            tray_item::TrayItem::new("DeckRemote", tray_item::IconSource::Resource(""))
        }) {
            Ok(mut tray) => {
                info!("System tray icon created");
                
                if let Err(e) = tray.add_label("DeckRemote Service") {
                    warn!("Failed to add tray label: {}", e);
                }
                
                if let Err(e) = tray.add_label("Running on :7394") {
                    warn!("Failed to add tray label: {}", e);
                }
                
                let _ = tray.inner_mut().add_separator();
                
                if let Err(e) = tray.add_menu_item("Exit", || {
                    info!("Exit requested from system tray");
                    std::process::exit(0);
                }) {
                    warn!("Failed to add exit menu item: {}", e);
                }

                // Keep the tray thread alive
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
            Err(e) => {
                warn!("Failed to create system tray icon: {}", e);
                warn!("Service will continue without system tray");
            }
        }
    });
}
