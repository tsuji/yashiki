mod app;
mod core;
mod effect;
mod event;
mod ipc;
mod layout;
mod macos;
mod pid;
mod platform;

use anyhow::{bail, Result};
use argh::FromArgs;
use ipc::IpcClient;
use tracing_subscriber::EnvFilter;
use yashiki_ipc::{Command, Direction, OutputDirection, OutputSpecifier, Response};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Yashiki - macOS tiling window manager
#[derive(FromArgs)]
struct Cli {
    #[argh(subcommand)]
    command: Option<SubCommand>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Start(StartCmd),
    Version(VersionCmd),
    Bind(BindCmd),
    Unbind(UnbindCmd),
    ListBindings(ListBindingsCmd),
    TagView(TagViewCmd),
    TagToggle(TagToggleCmd),
    TagViewLast(TagViewLastCmd),
    WindowMoveToTag(WindowMoveToTagCmd),
    WindowToggleTag(WindowToggleTagCmd),
    WindowFocus(WindowFocusCmd),
    WindowSwap(WindowSwapCmd),
    OutputFocus(OutputFocusCmd),
    OutputSend(OutputSendCmd),
    Retile(RetileCmd),
    LayoutSetDefault(LayoutSetDefaultCmd),
    LayoutSet(LayoutSetCmd),
    LayoutGet(LayoutGetCmd),
    LayoutCmd(LayoutCmdCmd),
    ListWindows(ListWindowsCmd),
    ListOutputs(ListOutputsCmd),
    GetState(GetStateCmd),
    FocusedWindow(FocusedWindowCmd),
    Exec(ExecCmd),
    ExecOrFocus(ExecOrFocusCmd),
    Quit(QuitCmd),
}

/// Start the yashiki daemon
#[derive(FromArgs)]
#[argh(subcommand, name = "start")]
struct StartCmd {}

/// Show version information
#[derive(FromArgs)]
#[argh(subcommand, name = "version")]
struct VersionCmd {}

/// Bind a hotkey to a command
#[derive(FromArgs)]
#[argh(subcommand, name = "bind")]
struct BindCmd {
    /// hotkey (e.g., alt-1, cmd-shift-h)
    #[argh(positional)]
    key: String,
    /// command and arguments to bind
    #[argh(positional, greedy)]
    action: Vec<String>,
}

/// Unbind a hotkey
#[derive(FromArgs)]
#[argh(subcommand, name = "unbind")]
struct UnbindCmd {
    /// hotkey to unbind
    #[argh(positional)]
    key: String,
}

/// List all hotkey bindings
#[derive(FromArgs)]
#[argh(subcommand, name = "list-bindings")]
struct ListBindingsCmd {}

/// Switch to specific tags (bitmask)
#[derive(FromArgs)]
#[argh(subcommand, name = "tag-view")]
struct TagViewCmd {
    /// output (display) ID or name
    #[argh(option)]
    output: Option<String>,
    /// tags bitmask (e.g., 1 for tag 1, 2 for tag 2, 3 for tags 1+2)
    #[argh(positional)]
    tags: u32,
}

/// Toggle visibility of tags (bitmask)
#[derive(FromArgs)]
#[argh(subcommand, name = "tag-toggle")]
struct TagToggleCmd {
    /// output (display) ID or name
    #[argh(option)]
    output: Option<String>,
    /// tags bitmask to toggle
    #[argh(positional)]
    tags: u32,
}

/// Switch to the previously viewed tags
#[derive(FromArgs)]
#[argh(subcommand, name = "tag-view-last")]
struct TagViewLastCmd {}

/// Move focused window to tags (bitmask)
#[derive(FromArgs)]
#[argh(subcommand, name = "window-move-to-tag")]
struct WindowMoveToTagCmd {
    /// tags bitmask
    #[argh(positional)]
    tags: u32,
}

/// Toggle tags on the focused window (bitmask)
#[derive(FromArgs)]
#[argh(subcommand, name = "window-toggle-tag")]
struct WindowToggleTagCmd {
    /// tags bitmask to toggle
    #[argh(positional)]
    tags: u32,
}

/// Focus a window in the specified direction
#[derive(FromArgs)]
#[argh(subcommand, name = "window-focus")]
struct WindowFocusCmd {
    /// direction: left, right, up, down, next, prev
    #[argh(positional)]
    direction: String,
}

/// Swap focused window with window in the specified direction
#[derive(FromArgs)]
#[argh(subcommand, name = "window-swap")]
struct WindowSwapCmd {
    /// direction: left, right, up, down, next, prev
    #[argh(positional)]
    direction: String,
}

/// Focus the next or previous display
#[derive(FromArgs)]
#[argh(subcommand, name = "output-focus")]
struct OutputFocusCmd {
    /// direction: next, prev
    #[argh(positional)]
    direction: String,
}

/// Send focused window to the next or previous display
#[derive(FromArgs)]
#[argh(subcommand, name = "output-send")]
struct OutputSendCmd {
    /// direction: next, prev
    #[argh(positional)]
    direction: String,
}

/// Re-apply the current layout
#[derive(FromArgs)]
#[argh(subcommand, name = "retile")]
struct RetileCmd {
    /// output (display) ID or name
    #[argh(option)]
    output: Option<String>,
}

/// Set the default layout engine
#[derive(FromArgs)]
#[argh(subcommand, name = "layout-set-default")]
struct LayoutSetDefaultCmd {
    /// layout engine name (e.g., tatami, byobu)
    #[argh(positional)]
    layout: String,
}

/// Set the layout engine for tags (current tag by default)
#[derive(FromArgs)]
#[argh(subcommand, name = "layout-set")]
struct LayoutSetCmd {
    /// tags bitmask, defaults to current tag
    #[argh(option)]
    tags: Option<u32>,
    /// output (display) ID or name
    #[argh(option)]
    output: Option<String>,
    /// layout engine name
    #[argh(positional)]
    layout: String,
}

/// Get the current layout engine
#[derive(FromArgs)]
#[argh(subcommand, name = "layout-get")]
struct LayoutGetCmd {
    /// tags bitmask, defaults to current layout
    #[argh(option)]
    tags: Option<u32>,
    /// output (display) ID or name
    #[argh(option)]
    output: Option<String>,
}

/// Send a command to the layout engine
#[derive(FromArgs)]
#[argh(subcommand, name = "layout-cmd")]
struct LayoutCmdCmd {
    /// layout command
    #[argh(positional)]
    cmd: String,
    /// command arguments
    #[argh(positional, greedy)]
    args: Vec<String>,
}

/// List all managed windows
#[derive(FromArgs)]
#[argh(subcommand, name = "list-windows")]
struct ListWindowsCmd {}

/// List all displays/outputs
#[derive(FromArgs)]
#[argh(subcommand, name = "list-outputs")]
struct ListOutputsCmd {}

/// Get current window manager state
#[derive(FromArgs)]
#[argh(subcommand, name = "get-state")]
struct GetStateCmd {}

/// Get the focused window ID
#[derive(FromArgs)]
#[argh(subcommand, name = "focused-window")]
struct FocusedWindowCmd {}

/// Execute a shell command
#[derive(FromArgs)]
#[argh(subcommand, name = "exec")]
struct ExecCmd {
    /// shell command to execute
    #[argh(positional)]
    command: String,
}

/// Focus an app if running, otherwise execute a command to launch it
#[derive(FromArgs)]
#[argh(subcommand, name = "exec-or-focus")]
struct ExecOrFocusCmd {
    /// application name to focus
    #[argh(option)]
    app_name: String,
    /// shell command to execute if app is not running
    #[argh(positional)]
    command: String,
}

/// Quit the yashiki daemon
#[derive(FromArgs)]
#[argh(subcommand, name = "quit")]
struct QuitCmd {}

fn main() -> Result<()> {
    let cli: Cli = argh::from_env();

    match cli.command {
        None => {
            // No subcommand - show help (simulate --help)
            let args: Vec<&str> = vec!["yashiki", "--help"];
            match Cli::from_args(&args[..1], &args[1..]) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e.output);
                }
            }
            Ok(())
        }
        Some(SubCommand::Start(_)) => {
            // Start daemon
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .init();

            tracing::info!("yashiki starting");
            app::App::run()
        }
        Some(SubCommand::Version(_)) => {
            println!("yashiki {}", VERSION);
            Ok(())
        }
        Some(subcmd) => run_cli(subcmd),
    }
}

fn run_cli(subcmd: SubCommand) -> Result<()> {
    let cmd = to_command(subcmd)?;
    let mut client = IpcClient::connect()?;
    let response = client.send(&cmd)?;

    match response {
        Response::Ok => {}
        Response::Error { message } => {
            eprintln!("Error: {}", message);
            std::process::exit(1);
        }
        Response::Windows { windows } => {
            for w in windows {
                println!(
                    "{}: {} - {} [tags={}, {}x{} @ ({},{})]{}",
                    w.id,
                    w.app_name,
                    w.title,
                    w.tags,
                    w.width,
                    w.height,
                    w.x,
                    w.y,
                    if w.is_focused { " *" } else { "" }
                );
            }
        }
        Response::Outputs { outputs } => {
            let mut sorted_outputs = outputs;
            sorted_outputs.sort_by_key(|o| o.id);
            for o in sorted_outputs {
                let main_marker = if o.is_main { " (main)" } else { "" };
                let focused_marker = if o.is_focused { " *" } else { "" };
                println!(
                    "{}: {} [{}x{} @ ({},{})]{}{}",
                    o.id, o.name, o.width, o.height, o.x, o.y, main_marker, focused_marker
                );
                println!("  visible_tags: {}", o.visible_tags);
            }
        }
        Response::State { state } => {
            println!("Visible tags: {}", state.visible_tags);
            println!("Focused window: {:?}", state.focused_window_id);
            println!("Window count: {}", state.window_count);
            println!("Default layout: {}", state.default_layout);
            println!(
                "Current layout: {}",
                state.current_layout.as_deref().unwrap_or("(default)")
            );
        }
        Response::Bindings { bindings } => {
            for b in bindings {
                println!("{} -> {}", b.key, b.action);
            }
        }
        Response::WindowId { id } => {
            if let Some(id) = id {
                println!("{}", id);
            } else {
                std::process::exit(1);
            }
        }
        Response::Layout { layout } => {
            println!("{}", layout);
        }
    }

    Ok(())
}

fn to_command(subcmd: SubCommand) -> Result<Command> {
    match subcmd {
        SubCommand::Start(_) | SubCommand::Version(_) => {
            unreachable!("handled in main")
        }
        SubCommand::Bind(cmd) => {
            if cmd.action.is_empty() {
                bail!("bind requires a command to bind");
            }
            let action = parse_command(&cmd.action)?;
            Ok(Command::Bind {
                key: cmd.key,
                action: Box::new(action),
            })
        }
        SubCommand::Unbind(cmd) => Ok(Command::Unbind { key: cmd.key }),
        SubCommand::ListBindings(_) => Ok(Command::ListBindings),
        SubCommand::TagView(cmd) => Ok(Command::TagView {
            tags: cmd.tags,
            output: parse_output_specifier(cmd.output),
        }),
        SubCommand::TagToggle(cmd) => Ok(Command::TagToggle {
            tags: cmd.tags,
            output: parse_output_specifier(cmd.output),
        }),
        SubCommand::TagViewLast(_) => Ok(Command::TagViewLast),
        SubCommand::WindowMoveToTag(cmd) => Ok(Command::WindowMoveToTag { tags: cmd.tags }),
        SubCommand::WindowToggleTag(cmd) => Ok(Command::WindowToggleTag { tags: cmd.tags }),
        SubCommand::WindowFocus(cmd) => Ok(Command::WindowFocus {
            direction: parse_direction(&cmd.direction)?,
        }),
        SubCommand::WindowSwap(cmd) => Ok(Command::WindowSwap {
            direction: parse_direction(&cmd.direction)?,
        }),
        SubCommand::OutputFocus(cmd) => Ok(Command::OutputFocus {
            direction: parse_output_direction(&cmd.direction)?,
        }),
        SubCommand::OutputSend(cmd) => Ok(Command::OutputSend {
            direction: parse_output_direction(&cmd.direction)?,
        }),
        SubCommand::Retile(cmd) => Ok(Command::Retile {
            output: parse_output_specifier(cmd.output),
        }),
        SubCommand::LayoutSetDefault(cmd) => Ok(Command::LayoutSetDefault { layout: cmd.layout }),
        SubCommand::LayoutSet(cmd) => Ok(Command::LayoutSet {
            tags: cmd.tags,
            output: parse_output_specifier(cmd.output),
            layout: cmd.layout,
        }),
        SubCommand::LayoutGet(cmd) => Ok(Command::LayoutGet {
            tags: cmd.tags,
            output: parse_output_specifier(cmd.output),
        }),
        SubCommand::LayoutCmd(cmd) => Ok(Command::LayoutCommand {
            cmd: cmd.cmd,
            args: cmd.args,
        }),
        SubCommand::ListWindows(_) => Ok(Command::ListWindows),
        SubCommand::ListOutputs(_) => Ok(Command::ListOutputs),
        SubCommand::GetState(_) => Ok(Command::GetState),
        SubCommand::FocusedWindow(_) => Ok(Command::FocusedWindow),
        SubCommand::Exec(cmd) => Ok(Command::Exec {
            command: cmd.command,
        }),
        SubCommand::ExecOrFocus(cmd) => Ok(Command::ExecOrFocus {
            app_name: cmd.app_name,
            command: cmd.command,
        }),
        SubCommand::Quit(_) => Ok(Command::Quit),
    }
}

fn parse_command(args: &[String]) -> Result<Command> {
    if args.is_empty() {
        bail!("No command provided");
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    match cmd {
        "bind" => {
            if rest.len() < 2 {
                bail!("Usage: bind <key> <command> [args...]");
            }
            let key = rest[0].clone();
            let action = parse_command(&rest[1..].to_vec())?;
            Ok(Command::Bind {
                key,
                action: Box::new(action),
            })
        }
        "unbind" => {
            if rest.is_empty() {
                bail!("Usage: unbind <key>");
            }
            Ok(Command::Unbind {
                key: rest[0].clone(),
            })
        }
        "list-bindings" => Ok(Command::ListBindings),
        "tag-view" => {
            let (output, rest) = parse_output_option(rest);
            if rest.is_empty() {
                bail!("Usage: tag-view [--output <id|name>] <tags>");
            }
            let tags: u32 = rest[0].parse()?;
            Ok(Command::TagView { tags, output })
        }
        "tag-toggle" => {
            let (output, rest) = parse_output_option(rest);
            if rest.is_empty() {
                bail!("Usage: tag-toggle [--output <id|name>] <tags>");
            }
            let tags: u32 = rest[0].parse()?;
            Ok(Command::TagToggle { tags, output })
        }
        "tag-view-last" => Ok(Command::TagViewLast),
        "window-move-to-tag" => {
            if rest.is_empty() {
                bail!("Usage: window-move-to-tag <tags>");
            }
            let tags: u32 = rest[0].parse()?;
            Ok(Command::WindowMoveToTag { tags })
        }
        "window-toggle-tag" => {
            if rest.is_empty() {
                bail!("Usage: window-toggle-tag <tags>");
            }
            let tags: u32 = rest[0].parse()?;
            Ok(Command::WindowToggleTag { tags })
        }
        "window-focus" => {
            if rest.is_empty() {
                bail!("Usage: window-focus <direction>");
            }
            let direction = parse_direction(&rest[0])?;
            Ok(Command::WindowFocus { direction })
        }
        "window-swap" => {
            if rest.is_empty() {
                bail!("Usage: window-swap <direction>");
            }
            let direction = parse_direction(&rest[0])?;
            Ok(Command::WindowSwap { direction })
        }
        "output-focus" => {
            if rest.is_empty() {
                bail!("Usage: output-focus <next|prev>");
            }
            let direction = parse_output_direction(&rest[0])?;
            Ok(Command::OutputFocus { direction })
        }
        "output-send" => {
            if rest.is_empty() {
                bail!("Usage: output-send <next|prev>");
            }
            let direction = parse_output_direction(&rest[0])?;
            Ok(Command::OutputSend { direction })
        }
        "retile" => {
            let (output, _rest) = parse_output_option(rest);
            Ok(Command::Retile { output })
        }
        "layout-set-default" => {
            if rest.is_empty() {
                bail!("Usage: layout-set-default <layout>");
            }
            Ok(Command::LayoutSetDefault {
                layout: rest[0].clone(),
            })
        }
        "layout-set" => {
            // Parse --tags and --output options if present
            let (output, rest) = parse_output_option(rest);
            let (tags, layout) = if rest.len() >= 3 && rest[0] == "--tags" {
                let tags: u32 = rest[1].parse()?;
                (Some(tags), rest[2].clone())
            } else if rest.is_empty() {
                bail!("Usage: layout-set [--tags <tags>] [--output <id|name>] <layout>");
            } else {
                (None, rest[0].clone())
            };
            Ok(Command::LayoutSet {
                tags,
                output,
                layout,
            })
        }
        "layout-get" => {
            // Parse --tags and --output options if present
            let (output, rest) = parse_output_option(rest);
            let tags = if rest.len() >= 2 && rest[0] == "--tags" {
                Some(rest[1].parse()?)
            } else {
                None
            };
            Ok(Command::LayoutGet { tags, output })
        }
        "layout-cmd" => {
            if rest.is_empty() {
                bail!("Usage: layout-cmd <cmd> [args...]");
            }
            Ok(Command::LayoutCommand {
                cmd: rest[0].clone(),
                args: rest[1..].to_vec(),
            })
        }
        "list-windows" => Ok(Command::ListWindows),
        "list-outputs" => Ok(Command::ListOutputs),
        "get-state" => Ok(Command::GetState),
        "focused-window" => Ok(Command::FocusedWindow),
        "exec" => {
            if rest.is_empty() {
                bail!("Usage: exec <command>");
            }
            Ok(Command::Exec {
                command: rest[0].clone(),
            })
        }
        "exec-or-focus" => {
            if rest.len() < 3 || rest[0] != "--app-name" {
                bail!("Usage: exec-or-focus --app-name <name> <command>");
            }
            let app_name = rest[1].clone();
            let command = rest[2].clone();
            Ok(Command::ExecOrFocus { app_name, command })
        }
        "quit" => Ok(Command::Quit),
        _ => bail!("Unknown command: {}", cmd),
    }
}

fn parse_direction(s: &str) -> Result<Direction> {
    match s.to_lowercase().as_str() {
        "left" => Ok(Direction::Left),
        "right" => Ok(Direction::Right),
        "up" => Ok(Direction::Up),
        "down" => Ok(Direction::Down),
        "next" => Ok(Direction::Next),
        "prev" => Ok(Direction::Prev),
        _ => bail!(
            "Unknown direction: {} (use left, right, up, down, next, prev)",
            s
        ),
    }
}

fn parse_output_direction(s: &str) -> Result<OutputDirection> {
    match s.to_lowercase().as_str() {
        "next" => Ok(OutputDirection::Next),
        "prev" => Ok(OutputDirection::Prev),
        _ => bail!("Unknown output direction: {} (use next or prev)", s),
    }
}

fn parse_output_specifier(s: Option<String>) -> Option<OutputSpecifier> {
    s.map(|s| {
        if let Ok(id) = s.parse::<u32>() {
            OutputSpecifier::Id(id)
        } else {
            OutputSpecifier::Name(s)
        }
    })
}

fn parse_output_option(args: &[String]) -> (Option<OutputSpecifier>, &[String]) {
    if args.len() >= 2 && args[0] == "--output" {
        let output = parse_output_specifier(Some(args[1].clone()));
        (output, &args[2..])
    } else {
        (None, args)
    }
}
