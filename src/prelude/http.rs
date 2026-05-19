use crate::interop::FfResult;
use crate::interpreter::Value;
use crate::members;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
struct Response {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Deserialize)]
struct RequestOpts {
    url: String,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    headers: Option<HashMap<String, String>>,
    #[serde(default)]
    body: Option<String>,
}

members! {
    "Http",
    get => |url: String| -> FfResult<Response> {
        send("GET", &url, &HashMap::new(), None).into()
    },
    post => |url: String, body: String| -> FfResult<Response> {
        send("POST", &url, &HashMap::new(), Some(&body)).into()
    },
    put => |url: String, body: String| -> FfResult<Response> {
        send("PUT", &url, &HashMap::new(), Some(&body)).into()
    },
    patch => |url: String, body: String| -> FfResult<Response> {
        send("PATCH", &url, &HashMap::new(), Some(&body)).into()
    },
    delete => |url: String| -> FfResult<Response> {
        send("DELETE", &url, &HashMap::new(), None).into()
    },
    head => |url: String| -> FfResult<Response> {
        send("HEAD", &url, &HashMap::new(), None).into()
    },
    request => |opts: RequestOpts| -> FfResult<Response> {
        let method = opts.method.as_deref().unwrap_or("GET");
        let headers = opts.headers.unwrap_or_default();
        send(method, &opts.url, &headers, opts.body.as_deref()).into()
    },
}

fn send(
    method: &str,
    url: &str,
    headers: &HashMap<String, String>,
    body: Option<&str>,
) -> Result<Response, String> {
    let mut req = ureq::request(method, url);
    for (name, value) in headers {
        req = req.set(name, value);
    }
    // 4xx/5xx surface as `Error::Status` but still carry the response — unwrap
    // it so callers can inspect `status` instead of getting an opaque error.
    let resp = match body {
        Some(b) => req.send_string(b),
        None => req.call(),
    };
    let resp = match resp {
        Ok(r) => r,
        Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(e.to_string()),
    };
    let status = resp.status();
    let mut header_map = HashMap::new();
    for name in resp.headers_names() {
        if let Some(value) = resp.header(&name) {
            header_map.insert(name, value.to_string());
        }
    }
    let body = if method.eq_ignore_ascii_case("HEAD") {
        String::new()
    } else {
        resp.into_string().map_err(|e| e.to_string())?
    };
    Ok(Response {
        status,
        headers: header_map,
        body,
    })
}
