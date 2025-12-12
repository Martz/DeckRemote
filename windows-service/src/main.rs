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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7394")
        .await
        .expect("Failed to bind to port 7394");

    println!("DeckRemote service running on http://0.0.0.0:7394");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn get_state(State(state): State<AppState>) -> Json<StateResponse> {
    let state = state.lock().unwrap();
    let blocked_keys: Vec<String> = state.blocked_keys.iter().cloned().collect();
    Json(StateResponse { blocked_keys })
}

async fn toggle_key(
    State(state): State<AppState>,
    Json(payload): Json<ToggleKeyRequest>,
) -> (StatusCode, Json<ToggleKeyResponse>) {
    let mut state = state.lock().unwrap();
    let was_blocked = state.blocked_keys.contains(&payload.key);
    
    if was_blocked {
        state.blocked_keys.remove(&payload.key);
    } else {
        state.blocked_keys.insert(payload.key.clone());
    }
    
    let now_blocked = !was_blocked;
    
    (
        StatusCode::OK,
        Json(ToggleKeyResponse {
            success: true,
            blocked: now_blocked,
        }),
    )
}

#[cfg(windows)]
fn install_keyboard_hook(state: AppState) {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    static mut GLOBAL_STATE: Option<AppState> = None;
    
    unsafe {
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
            let mut state = state.lock().unwrap();
            state.hook_handle = Some(hook);
            println!("Keyboard hook installed successfully");
        } else {
            eprintln!("Failed to install keyboard hook");
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
                let state = global_state.lock().unwrap();
                
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
        }
    }
    
    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(windows)]
fn setup_tray() {
    std::thread::spawn(|| {
        let mut tray = tray_item::TrayItem::new(
            "DeckRemote",
            tray_item::IconSource::Resource("tray-icon"),
        )
        .unwrap_or_else(|_| {
            tray_item::TrayItem::new("DeckRemote", tray_item::IconSource::Resource("")).unwrap()
        });

        tray.add_label("DeckRemote Service").unwrap();
        tray.add_label("Running on :7394").unwrap();
        
        tray.inner_mut().add_separator().unwrap();
        
        tray.add_menu_item("Exit", || {
            std::process::exit(0);
        })
        .unwrap();

        // Keep the tray thread alive
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
