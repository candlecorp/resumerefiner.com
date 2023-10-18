mod wick {
    wick_component::wick_import!();
}
use std::collections::HashMap;

use wick::*;

fn get_header_value(headers: &HashMap<String, Vec<String>>, header_name: &str) -> Option<String> {
    headers
        .get(header_name)
        .and_then(|header_values| header_values.get(0).cloned())
}

fn build_redirect_response(location: &str) -> types::http::HttpResponse {
    let mut headers: HashMap<String, Vec<_>> = HashMap::new();

    headers.insert("location".to_string(), vec![location.to_string()]);

    types::http::HttpResponse {
        version: types::http::HttpVersion::Http11,
        status: types::http::StatusCode::Found,
        headers: headers,
    }
}

fn build_response(status: types::http::StatusCode) -> types::http::HttpResponse {
    let mut headers: HashMap<String, Vec<_>> = HashMap::new();

    headers.insert(
        "content-type".to_string(),
        vec!["application/json".to_string()],
    );

    types::http::HttpResponse {
        version: types::http::HttpVersion::Http11,
        status: status,
        headers: headers,
    }
}

fn build_streaming_response(status: types::http::StatusCode) -> types::http::HttpResponse {
    let mut headers: HashMap<String, Vec<_>> = HashMap::new();

    headers.insert(
        "content-type".to_string(),
        vec!["text/event-stream".to_string()],
    );

    types::http::HttpResponse {
        version: types::http::HttpVersion::Http11,
        status: status,
        headers: headers,
    }
}

fn build_error_streaming_body(message: String) -> Vec<types::http::HttpEvent> {
    vec![
        types::http::HttpEvent {
            event: "message".to_string(),
            data: message,
            id: "".to_string(),
            retry: None,
        },
        types::http::HttpEvent {
            event: "message".to_string(),
            data: "[DONE]".to_string(),
            id: "".to_string(),
            retry: None,
        },
    ]
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl check_usage::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = check_usage::Inputs;
    type Outputs = check_usage::Outputs;
    type Config = check_usage::Config;
    async fn check_usage(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let (Some(request), Some(body)) =
            (inputs.request.next().await, inputs.body.next().await)
        {
            let request = request.decode()?;
            let _body = body.decode()?;
            let email = get_header_value(&request.headers, "x-oidc-email");

            if email.is_none() {
                let response = build_response(types::http::StatusCode::Unauthorized);
                outputs.response.send(&response);
                let body = types::DailyUsage {
                    daily_usage_value: 0,
                    entitlement_value: 0,
                    entitlement_name: "tokens".to_string(),
                };
                outputs.body.send(&body);
                return Ok(());
            }

            let email = email.unwrap();

            let mut get_usage_stream = ctx.provided().db_client.get_daily_usage(
                db_client::get_daily_usage::Config::default(),
                db_client::get_daily_usage::Request {
                    entitlement: "tokens".to_string(),
                    email: email.clone(),
                },
            )?;

            while let Some(daily_usage) = get_usage_stream.output.next().await {
                let daily_usage: types::DailyUsage = daily_usage.decode()?;
                println!("daily_usage: {:?}", daily_usage);
                let response = build_response(types::http::StatusCode::Ok);
                outputs.response.send(&response);
                outputs.body.send(&daily_usage);
                outputs.response.done();
                outputs.body.done();
                return Ok(());
            }

            println!("no usage found");
            let response = build_response(types::http::StatusCode::Ok);
            outputs.response.send(&response);
            let body = types::DailyUsage {
                daily_usage_value: 0,
                entitlement_value: 100,
                entitlement_name: "tokens".to_string(),
            };
            outputs.body.send(&body);
            return Ok(());
        }
        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl refine::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = refine::Inputs;
    type Outputs = refine::Outputs;
    type Config = refine::Config;
    async fn refine(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let (Some(request), Some(body)) =
            (inputs.request.next().await, inputs.body.next().await)
        {
            let request = request.decode()?;
            let body: types::RefineRequest = body.decode()?;
            let email = get_header_value(&request.headers, "x-oidc-email");

            if body.job_requirement == "" || body.extended_resume == "" {
                let response = build_streaming_response(types::http::StatusCode::BadRequest);
                outputs.response.send(&response);
                let body = build_error_streaming_body(
                    "Missing job requirement or extended resume".to_string(),
                );
                for event in body {
                    outputs.body.send(&event);
                }
                outputs.response.done();
                outputs.body.done();
                return Ok(());
            }

            if email.is_none() {
                let response = build_streaming_response(types::http::StatusCode::Unauthorized);
                outputs.response.send(&response);
                let body = build_error_streaming_body("Authentication error".to_string());
                for event in body {
                    outputs.body.send(&event);
                }
                outputs.response.done();
                outputs.body.done();
                return Ok(());
            }

            let email = email.unwrap();

            let mut get_usage_stream = ctx.provided().db_client.get_daily_usage(
                db_client::get_daily_usage::Config::default(),
                db_client::get_daily_usage::Request {
                    entitlement: "tokens".to_string(),
                    email: email.clone(),
                },
            )?;

            while let Some(daily_usage) = get_usage_stream.output.next().await {
                let daily_usage: types::DailyUsage = daily_usage.decode()?;
                println!("daily_usage: {:?}", daily_usage);
                if daily_usage.daily_usage_value >= daily_usage.entitlement_value {
                    let response = build_streaming_response(types::http::StatusCode::Ok);
                    outputs.response.send(&response);
                    let body = build_error_streaming_body(
                        "You have exceeded your daily usage".to_string(),
                    );
                    for event in body {
                        outputs.body.send(&event);
                    }
                    outputs.response.done();
                    outputs.body.done();
                    return Ok(());
                }
            }

            let mut get_refine_response = ctx.provided().openai_client.refine(
                openai_client::refine::Config::default(),
                openai_client::refine::Request {
                    job_requirement: body.job_requirement.clone(),
                    extended_resume: body.extended_resume.clone(),
                },
            )?;

            while let Some(response) = get_refine_response.response.next().await {
                let response = response.decode()?;
                println!("response: {:?}", response);

                let out_response = build_streaming_response(types::http::StatusCode::Ok);
                outputs.response.send(&out_response);

                while let Some(body) = get_refine_response.body.next().await {
                    let body = body.decode()?;
                    println!("body: {:?}", body);
                    outputs.body.send(&body);
                }
                while let Some(tokens) = get_refine_response.tokens.next().await {
                    let tokens = tokens.decode()?;
                    println!("tokens: {:?}", tokens);

                    let mut create_usage_stream = ctx.provided().db_client.upsert_token_usage(
                        db_client::upsert_token_usage::Config::default(),
                        db_client::upsert_token_usage::Request {
                            email: email.clone(),
                            tokens: tokens,
                        },
                    )?;

                    while let Some(daily_usage) = create_usage_stream.output.next().await {
                        let daily_usage = daily_usage.decode()?;
                        println!("daily_usage: {:?}", daily_usage);
                    }
                }
                outputs.response.done();
                outputs.body.done();
                return Ok(());
            }
        }
        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl login::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = login::Inputs;
    type Outputs = login::Outputs;
    type Config = login::Config;
    async fn login(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        let entitlements = ctx.root_config().entitlements.clone();
        while let Some(request) = inputs.request.next().await {
            let request = request.decode()?;
            let email = get_header_value(&request.headers, "x-oidc-email");

            if email.is_none() {
                let response = build_response(types::http::StatusCode::Unauthorized);
                outputs.response.send(&response);
                let body = wick_component::to_value("".to_string())?;
                outputs.body.send(&body);
                return Ok(());
            }

            let email = email.unwrap();
            let mut get_user_stream = ctx.provided().db_client.get_user(
                db_client::get_user::Config::default(),
                db_client::get_user::Request {
                    email: email.clone(),
                },
            )?;

            let mut exists = false;
            while let Some(user_response) = get_user_stream.output.next().await {
                let _user = user_response.decode()?;
                println!("exists user: {:?}", _user);
                exists = true;
            }

            if !exists {
                let mut create_user_stream = ctx.provided().db_client.create_user(
                    db_client::create_user::Config::default(),
                    db_client::create_user::Request {
                        email: email.clone(),
                    },
                )?;
                while let Some(user_response) = create_user_stream.output.next().await {
                    let user = user_response.decode()?;
                    println!("new user: {:?}", user);

                    for entitlement in entitlements.clone() {
                        let mut create_entitlement_stream =
                            ctx.provided().db_client.add_entitlement(
                                db_client::add_entitlement::Config::default(),
                                db_client::add_entitlement::Request {
                                    email: email.clone(),
                                    entitlement: entitlement.name.to_string(),
                                    value: entitlement.value,
                                    value_unit: "".to_string(),
                                },
                            )?;
                        while let Some(entitlement_response) =
                            create_entitlement_stream.output.next().await
                        {
                            let _entitlement = entitlement_response.decode()?;
                            println!("new entitlement: {:?}", _entitlement);
                        }
                    }
                }
            }

            let response = build_redirect_response("/");
            outputs.response.send(&response);
            let body = wick_component::to_value("".to_string())?;
            outputs.body.send(&body);
        }
        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl confirm_star::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = confirm_star::Inputs;
    type Outputs = confirm_star::Outputs;
    type Config = confirm_star::Config;
    async fn confirm_star(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let Some(request) = inputs.request.next().await {
            let request = request.decode()?;
            let email = get_header_value(&request.headers, "x-oidc-email");

            if email.is_none() {
                let response = build_response(types::http::StatusCode::Ok);
                outputs.response.send(&response);
                let body = types::ConfirmStar {
                    status: false,
                    message: "Please login with your Github Account".to_string(),
                };
                outputs.body.send(&body);
                return Ok(());
            }

            let email = email.unwrap();
            println!("email: {:?}", email);
            let mut get_github_username_stream = ctx.provided().db_client.get_github_username(
                db_client::get_github_username::Config::default(),
                db_client::get_github_username::Request {
                    email: email.clone(),
                },
            )?;

            let mut github_user: Option<String> = None;
            while let Some(github_username) = get_github_username_stream.output.next().await {
                let github_username: types::GithubUsername = github_username.decode()?;
                println!("exists user: {:?}", github_username);
                github_user = github_username.github_username;
            }

            if github_user.is_none() {
                let response = build_response(types::http::StatusCode::Ok);
                outputs.response.send(&response);
                let body = types::ConfirmStar {
                    status: false,
                    message: "Please log out and then login with your Github Account".to_string(),
                };
                outputs.body.send(&body);
                return Ok(());
            }

            let github_user = github_user.unwrap();
            println!("github_user: {:?}", github_user);

            let mut get_stargazer_stream = ctx.provided().candle_cloud_client.get_stargazer(
                candle_cloud_client::get_stargazer::Config::default(),
                candle_cloud_client::get_stargazer::Request {
                    github_user: github_user.clone(),
                },
            )?;
            while let (Some(response), Some(body)) = (
                get_stargazer_stream.response.next().await,
                get_stargazer_stream.body.next().await,
            ) {
                let response = response.decode()?;
                println!("stargazer response: {:?}", response);

                let stargazer = body.decode()?;
                println!("stargazer body: {:?}", stargazer);

                let response = build_response(types::http::StatusCode::Ok);
                outputs.response.send(&response);

                let mut resp_body: types::ConfirmStar;

                if stargazer.is_stargazer == true {
                    let mut create_achievement_response =
                        ctx.provided().db_client.create_achievement(
                            db_client::create_achievement::Config::default(),
                            db_client::create_achievement::Request {
                                email: email.clone(),
                                achievement: "stargazer".to_string(),
                            },
                        )?;

                    while let Some(achievement_response) =
                        create_achievement_response.output.next().await
                    {
                        let _achievement = achievement_response.decode()?;
                        println!("new achievement: {:?}", _achievement);

                        let mut update_entitlement_response =
                            ctx.provided().db_client.update_entitlement(
                                db_client::update_entitlement::Config::default(),
                                db_client::update_entitlement::Request {
                                    email: email.clone(),
                                    entitlement: "tokens".to_string(),
                                    value: 10000,
                                },
                            )?;
                        while let Some(entitlement_response) =
                            update_entitlement_response.output.next().await
                        {
                            let _entitlement = entitlement_response.decode()?;
                            println!("new entitlement: {:?}", _entitlement);
                        }
                    }

                    resp_body = types::ConfirmStar {
                        status: true,
                        message: "Success! You are a stargazer! Thank you for starring our repo!"
                            .to_string(),
                    };
                } else {
                    resp_body = types::ConfirmStar {
                        status: false,
                        message: "You are not a stargazer. Please ensure you have starred our repo and are logged into this site with your Github account.".to_string(),
                    }
                };
                outputs.body.send(&resp_body);
            }
        }
        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}

#[cfg_attr(target_family = "wasm",async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl stargazer::Operation for Component {
    type Error = anyhow::Error;
    type Inputs = stargazer::Inputs;
    type Outputs = stargazer::Outputs;
    type Config = stargazer::Config;
    async fn stargazer(
        mut inputs: Self::Inputs,
        mut outputs: Self::Outputs,
        ctx: Context<Self::Config>,
    ) -> anyhow::Result<()> {
        while let Some(request) = inputs.request.next().await {
            let request = request.decode()?;
            let email = get_header_value(&request.headers, "x-oidc-email");

            if email.is_none() {
                let response = build_response(types::http::StatusCode::Ok);
                outputs.response.send(&response);
                let body = types::Stargazer {
                    is_stargazer: false,
                };
                outputs.body.send(&body);
                return Ok(());
            }

            let email = email.unwrap();
            println!("email: {:?}", email);
            let mut get_stargazer_stream = ctx.provided().db_client.get_achievement(
                db_client::get_achievement::Config::default(),
                db_client::get_achievement::Request {
                    email: email.clone(),
                    achievement: "stargazer".to_string(),
                },
            )?;

            while let Some(stargazer) = get_stargazer_stream.output.next().await {
                let stargazer = stargazer.decode()?;
                println!("stargazer exists: {:?}", stargazer);
                let response = build_response(types::http::StatusCode::Ok);
                outputs.response.send(&response);
                let body = types::Stargazer { is_stargazer: true };
                outputs.body.send(&body);
                return Ok(());
            }
        }
        let response = build_response(types::http::StatusCode::Ok);
        outputs.response.send(&response);
        let body = types::Stargazer {
            is_stargazer: false,
        };
        outputs.body.send(&body);
        outputs.response.done();
        outputs.body.done();
        Ok(())
    }
}
