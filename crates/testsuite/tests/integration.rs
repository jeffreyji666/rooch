use anyhow::{bail, Result};
use clap::Parser;
use cucumber::{given, then, World as _};
use jpst::TemplateContext;
use rooch::RoochCli;
use rooch_server::Service;
use serde_json::Value;
use tracing::info;

#[derive(cucumber::World, Debug, Default)]
struct World {
    service: Option<Service>,
    tpl_ctx: Option<TemplateContext>,
}

#[given(expr = "a server")] // Cucumber Expression
async fn start_server(w: &mut World) {
    let mut service = Service::new();
    service.start().await.unwrap();

    w.service = Some(service);
}

#[then(regex = r#"cmd: "(.*)?""#)]
async fn run_cmd(world: &mut World, args: String) {
    let mut cmd = assert_cmd::Command::cargo_bin("rooch").unwrap();
    // Discard test data
    cmd.env("TEST_ENV", "true");

    if world.tpl_ctx.is_none() {
        world.tpl_ctx = Some(TemplateContext::new());
    }
    let tpl_ctx = world.tpl_ctx.as_mut().unwrap();
    let args = eval_command_args(tpl_ctx, args);

    let mut args = split_string_with_quotes(&args).expect("Invalid commands");
    let cmd_name = args[0].clone();
    args.insert(0, "rooch".to_string());
    let opts: RoochCli = RoochCli::parse_from(args);
    let output = rooch::run_cli(opts)
        .await
        .expect("CLI should run successfully.");

    // info!("cmd output: {:?}", output);
    let result_json: Value =
        serde_json::from_str(&output).expect("json parse error from cli output.");
    tpl_ctx.entry(cmd_name).append(result_json);
}

#[then(regex = r#"assert: "([^"]*)""#)]
async fn assert_output(world: &mut World, args: String) {
    assert!(world.tpl_ctx.is_some(), "tpl_ctx is none");
    let args = eval_command_args(world.tpl_ctx.as_ref().unwrap(), args);
    let parameters = args.split_whitespace().collect::<Vec<_>>();

    for chunk in parameters.chunks(3) {
        let first = chunk.get(0).cloned();
        let op = chunk.get(1).cloned();
        let second = chunk.get(2).cloned();

        info!("assert value: {:?} {:?} {:?}", first, op, second);

        match (first, op, second) {
            (Some(first), Some(op), Some(second)) => match op {
                "==" => assert_eq!(first, second),
                "!=" => assert_ne!(first, second),
                _ => panic!("unsupported operator"),
            },
            _ => panic!("expected 3 arguments: first [==|!=] second"),
        }
    }
    info!("assert ok!");
}

fn eval_command_args(ctx: &TemplateContext, args: String) -> String {
    // info!("args: {}", args);
    let args = args.replace("\\\"", "\"");
    let eval_args = jpst::format_str!(&args, ctx);
    // info!("eval args:{}", eval_args);
    eval_args
}

/// Split a string into a vector of strings, splitting on spaces, but ignoring spaces inside quotes.
/// And quotes will alse be removed.
fn split_string_with_quotes(s: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                // Skip the quote
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_quotes {
        bail!("Mismatched quotes")
    }

    if !current.is_empty() {
        result.push(current);
    }

    Ok(result)
}

#[tokio::main]
async fn main() {
    World::cucumber()
        .run_and_exit("./features/cmd.feature")
        .await;
}
