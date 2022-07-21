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
        .get("/", |_, _| Response::ok(constants::HOME_MSG))
        .get_async("/:sns_name", |_, ctx| async move {
            match ctx.param("sns_name") {
                Some(sns_name) => {
                    let url = get_name_url(sns_name).await;
                    let response = if let Ok(url) = url {
                        Response::redirect(url)
                    } else {
                        Response::redirect(Url::parse(constants::ERROR_URL).unwrap())
                    };
                    return response;
                }
                None => return Response::redirect(Url::parse(constants::ERROR_URL).unwrap()),
            };
        })
        .run(req, env)
        .await
}
