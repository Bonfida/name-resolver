mod constants;
mod derivation;
mod utils;

use {derivation::get_name_url, worker::*};

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("Visit https://bonfida.org"))
        .get_async("/:sns_name", |_, ctx| async move {
            match ctx.param("sns_name") {
                Some(sns_name) => {
                    let name_url = get_name_url(sns_name).await;
                    let response = if let Ok(name_url) = name_url {
                        Response::redirect(Url::parse(&name_url).unwrap())
                    } else {
                        Response::error("Invalid domain record", 400)
                    };
                    return response;
                }
                None => return Response::error("Bad Request", 400),
            };
        })
        .run(req, env)
        .await
}
