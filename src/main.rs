mod config;


use std::borrow::Cow;
use std::convert::Infallible;
use std::path::PathBuf;

use askama::Template;
use clap::Parser;
use form_urlencoded;
use hyper::{Body, Method, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use once_cell::sync::OnceCell;
use percent_encoding::percent_decode_str;
use tokio::sync::RwLock;
use tracing::error;

use crate::config::{CONFIG, CONFIG_PATH};


#[derive(Parser)]
struct Opts {
    #[arg(default_value = "config.toml")]
    pub config_path: PathBuf,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;


static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();


fn decode_path_parts(path: &str) -> Vec<String> {
    path
        .split('/')
        .map(|piece| percent_decode_str(piece).decode_utf8_lossy().into_owned())
        .collect()
}

fn return_500() -> Result<Response<Body>, Infallible> {
    Ok(
        Response::builder()
            .status(500)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Body::from("500 Internal Server Error"))
            .expect("failed to construct HTTP 500 response")
    )
}

async fn handle_plaintext_response<S: Into<String>>(status_code: u16, text_body: S) -> Result<Response<Body>, Infallible> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(text_body.into()))
        .or_else(|e| {
            error!("failed to construct plain-text response: {}", e);
            return_500()
        })
}

async fn handle_404(_request: Request<Body>) -> Result<Response<Body>, Infallible> {
    handle_plaintext_response(404, "404 Not Found").await
}

async fn handle_index(_request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let template = IndexTemplate;
    let rendered = match template.render() {
        Ok(r) => r,
        Err(e) => {
            error!("failed to render template: {}", e);
            return return_500();
        },
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(rendered))
        .or_else(|e| {
            error!("failed to construct template response: {}", e);
            return_500()
        })
}

async fn handle_static(request: Request<Body>, file_name: &str) -> Result<Response<Body>, Infallible> {
    let (mime_type, bytes) = if file_name == "script.js" {
        ("text/javascript", include_bytes!("../static/script.js"))
    } else if file_name == "script.ts" {
        ("application/typescript", include_bytes!("../static/script.ts"))
    } else if file_name == "script.js.map" {
        ("application/json", include_bytes!("../static/script.js.map"))
    } else {
        return handle_404(request).await;
    };

    Response::builder()
        .status(200)
        .header("Content-Type", mime_type)
        .body(Body::from(bytes))
        .or_else(|e| {
            error!("failed to construct static file response: {}", e);
            return_500()
        })
}

async fn handle_400_missing_parameter(param_name: &str) -> Result<Response<Body>, Infallible> {
    handle_plaintext_response(
        400,
        format!("missing required parameter {:?}", param_name),
    ).await
}

async fn handle_400_wrong_parameter(param_name: &str, value: &str) -> Result<Response<Body>, Infallible> {
    handle_plaintext_response(
        400,
        format!("required parameter {:?} has invalid value {:?}", param_name, value),
    ).await
}

async fn get_required_parameter<'a>(query_pairs: &'a [(Cow<'a, str>, Cow<'a, str>)], key: &str) -> Result<&'a Cow<'a, str>, Result<Response<Body>, Infallible>> {
    let val_opt = query_pairs
        .iter()
        .filter(|(k, _v)| k == key)
        .map(|(_k, v)| v)
        .last();
    match val_opt {
        Some(v) => Ok(v),
        None => Err(handle_400_missing_parameter(key).await),
    }
}

async fn handle_table(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let query_pairs: Vec<(Cow<str>, Cow<str>)> = if let Some(query) = request.uri().query() {
        form_urlencoded::parse(query.as_bytes())
            .collect()
    } else {
        Vec::new()
    };

    // what are we querying?
    let objtype = match get_required_parameter(&query_pairs, "objtype").await {
        Ok(ot) => ot,
        Err(resp) => return resp,
    };
    if objtype != "hosts" && objtype != "services" {
        return handle_400_wrong_parameter("objtype", objtype).await;
    }

    // what's the filter?
    let filter = match get_required_parameter(&query_pairs, "filter").await {
        Ok(f) => f,
        Err(resp) => return resp,
    };

    // build Icinga API JSON body
    let api_body = serde_json::json!({
        "filter": filter,
    });

    let icinga_config = {
        let config_guard = CONFIG
            .get().expect("CONFIG not set?!")
            .read().await;
        config_guard.icinga_api.clone()
    };
    let icinga_url_path = format!("objects/{}", objtype);
    let icinga_url = match icinga_config.base_url.join("") {
        Ok(u) => u,
        Err(e) => {
            error!(
                "failed to append object type-specific path {:?} to Icinga API base URL {:?}: {}",
                icinga_url_path, icinga_config.base_url, e,
            );
            return return_500();
        },
    };

    // contact Icinga
    // TODO: SSL cert handling
    let client = CLIENT.get().expect("CLIENT not set?!");
    let response_res = client
        .request(Method::POST, icinga_url)
        .basic_auth(&icinga_config.username, Some(&icinga_config.password))
        .header("X-HTTP-Method-Override", "GET")
        .body(serde_json::to_string(&api_body).expect("cannot serialize serde_json::Value to JSON?!"))
        .send().await;

    let template = IndexTemplate;
    let rendered = match template.render() {
        Ok(r) => r,
        Err(e) => {
            error!("failed to render template: {}", e);
            return return_500();
        },
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(rendered))
        .or_else(|e| {
            error!("failed to construct template response: {}", e);
            return_500()
        })
}

async fn handle_http(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut path_parts = decode_path_parts(request.uri().path());
    while path_parts.len() > 0 && path_parts[0].len() == 0 {
        path_parts.remove(0);
    }

    if path_parts.len() == 0 {
        handle_index(request).await
    } else if &path_parts == &["table"] {
        handle_table(request).await
    } else if path_parts.len() == 2 && path_parts[0] == "static" {
        handle_static(request, &path_parts[1]).await
    } else {
        handle_404(request).await
    }
}


#[tokio::main]
async fn main() {
    // parse command line
    let opts = Opts::parse();

    // set up tracing
    let (stdout_non_blocking, _guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(false)
        .finish(std::io::stdout());
    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format())
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(stdout_non_blocking)
        .init();

    // store config path
    CONFIG_PATH.set(opts.config_path).expect("CONFIG_PATH already set?!");

    // load config
    let config = config::load().expect("failed to load config");
    let listen_socket_address = config.http_server.listen_socket_address;
    CONFIG.set(RwLock::new(config)).expect("CONFIG already set?!");

    // create HTTP client
    let client = reqwest::Client::builder()
        .build()
        .expect("failed to initialize HTTP client");
    CLIENT.set(client).expect("CLIENT already set?!");

    // create HTTP server
    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_http))
    });
    let server = Server::bind(&listen_socket_address).serve(make_service);

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}
