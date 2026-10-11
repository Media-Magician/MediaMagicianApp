#[cfg(any(target_os = "linux", target_os = "windows"))]
use tauri::{AppHandle, Manager, Runtime};

#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::sync::Mutex;

#[cfg(any(target_os = "linux", target_os = "windows"))]
#[derive(Default)]
pub struct AuthListenerState(Mutex<Option<u16>>);

#[cfg(target_os = "android")]
#[tauri::command]
pub fn start_auth_listener() -> Result<(), String> {
  Ok(())
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
#[tauri::command]
pub fn start_auth_listener<R: Runtime>(app: AppHandle<R>) -> Result<u16, String> {
    use std::{thread, time::Duration};
    use tiny_http::Server;
    use url::Url;

    let state = app.state::<AuthListenerState>();
    let mut active_port = state.0.lock()
        .map_err(|error| format!("auth listener state is unavailable: {error}"))?;

    if let Some(port) = *active_port {
        return Ok(port);
    }

    let server = Server::http("127.0.0.1:0").map_err(|error| error.to_string())?;
    let port = server.server_addr().to_ip()
        .ok_or_else(|| "auth listener did not bind to an IP address".to_string())?
        .port();

    let listener_app = app.clone();
    thread::Builder::new()
        .name("auth-listener".into())
        .spawn(move || {
            loop {
                let request = match server.recv() {
                    Ok(request) => request,
                    Err(error) => {
                        eprintln!("Auth listener failed to receive a request: {error}");
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                };

                let requestUrl = Url::parse(&format!("http://127.0.0.1{}", request.url()));
                let magicLink = requestUrl.as_ref().ok().and_then(|url| {
                    if request.method() == &tiny_http::Method::Get && url.path() == "/open/auth" {
                        url.query_pairs()
                            .find(|(key, _)| key == "magicLink")
                            .map(|(_, value)| value.into_owned())
                    } else {
                        None
                    }
                });
                // let magicLinkHost = requestUrl.ok().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or("beta.mediamagician.app".into());
                let targetUrl = magicLink.and_then(|link| {
                    Url::parse(&link).ok().filter(|url| matches!(url.scheme(), "http" | "https"))
                });

                let Some(targetUrl) = targetUrl else {
                    respond(
                        request,
                        "beta.mediamagician.app".into(),
                        "error",
                        "Invalid authentication link. Please try again.",
                    );

                    continue;
                };

                let targetUrlClone = targetUrl.clone();
                let magicLinkHost = targetUrlClone.host_str().unwrap_or("beta.mediamagician.app");

                let Some(window) = listener_app.get_webview_window("main") else {
                    respond(
                        request,
                        magicLinkHost,
                        "error",
                        "The app window is unavailable. Please try again.",
                    );

                    continue;
                };

                let nav = window.navigate(targetUrl);
                if let Err(error) = nav {
                    eprintln!("Failed to navigate to the authentication link: {error}");
                    respond(
                        request,
                        magicLinkHost,
                        "error",
                        "Could not open the authentication link. Please try again.",
                    );

                    continue;
                };

                respond(
                    request,
                    magicLinkHost,
                    "success",
                    "Authentication link opened. You can return to the app.",
                );

                break;
            }

            drop(server);
            match listener_app.state::<AuthListenerState>().0.lock() {
                Ok(mut active_port) if *active_port == Some(port) => *active_port = None,
                Ok(_) => {}
                Err(error) => eprintln!("Could not reset auth listener state: {error}"),
            }
        })
        .map_err(|error| format!("could not start auth listener: {error}"))?;

    *active_port = Some(port);
    Ok(port)
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn respond(request: tiny_http::Request, host: &str, notifType: &str, body: &str) {
    use std::str::FromStr;
    use tiny_http::{Header, HeaderField, StatusCode, Response};
    use ascii::AsciiString;

    let location = Header {
        field: HeaderField::from_str("Location").unwrap(),
        value: AsciiString::from_ascii(format!("https://{}?{}={}", host, notifType, body)).unwrap(),
    };
    
    if let Err(error) = request.respond(Response::from_string("").with_status_code(StatusCode(302)).with_header(location)) {
        eprintln!("Failed to respond to auth callback: {error}");
    };
}
