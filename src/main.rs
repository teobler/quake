use std::error::Error;
use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;

use clap::Parser;

use quake_core::entry::entry_defines::EntryDefines;
use quake_core::parser::action_parser::ActionDefine;
use quake_core::QuakeConfig;
use quake_tui::tui_main_loop;

use crate::server::start_server;

pub mod action;
pub mod cli;
mod errors;
pub mod helper;
pub mod server;
pub mod tui;

#[derive(Parser)]
#[clap(version = "0.0.1", author = "Phodal HUANG<h@phodal.com>")]
pub struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    /// init project
    Init(Init),
    /// command for CRUD entries
    Cmd(Command),
    /// web server for run
    Server(WebServer),
    /// Terminal UI
    Tui(Terminal),
}

#[derive(Parser)]
pub struct Terminal {}

#[derive(Parser)]
pub struct WebServer {}

#[derive(Parser)]
pub struct Init {
    /// init by path
    #[clap(short, long, default_value = ".")]
    path: String,
}

#[derive(Parser, Debug)]
pub struct Command {
    /// config path
    #[clap(short, long, default_value = ".quake.yaml")]
    config: String,
    /// like `todo.add: hello world`
    #[clap(short, long)]
    input: String,
    /// config the editor
    #[clap(short, long, default_value = "")]
    editor: String,
}

fn load_config(cmd: &Command) -> Result<QuakeConfig, Box<dyn Error>> {
    let content = fs::read_to_string(&cmd.config)?;
    let mut conf: QuakeConfig = serde_yaml::from_str(content.as_str())?;

    if !cmd.editor.is_empty() {
        conf.editor = cmd.editor.clone();
    }

    Ok(conf)
}

fn main() {
    let mut stdout = stdout();
    let opts: Opts = Opts::parse();
    if let Err(err) = process_cmd(opts) {
        stdout.write(format!("{:?}", err).as_bytes()).unwrap();
    }
}

pub fn process_cmd(opts: Opts) -> Result<(), Box<dyn Error>> {
    match opts.cmd {
        SubCommand::Init(init) => init_projects(init)?,
        SubCommand::Cmd(cmd) => {
            let conf = load_config(&cmd)?;
            if cmd.input.len() > 0 {
                let expr = ActionDefine::from(cmd.input.as_str())?;
                cli::action(expr, conf)?
            }
        }
        SubCommand::Server(_) => {
            start_server()?;
        }
        SubCommand::Tui(_) => {
            tui_main_loop()?;
        }
    }

    Ok(())
}

fn init_projects(config: Init) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.path)?;

    let path = PathBuf::from(&config.path).join(".quake.yaml");
    let define = PathBuf::from(&config.path).join("entries-define.yaml");

    let config = QuakeConfig {
        workspace: config.path.clone(),
        editor: "".to_string(),
        search_url: "http://127.0.0.1:7700".to_string(),
        server_location: "web".to_string(),
    };

    fs::write(path, serde_yaml::to_string(&config)?)?;

    let todo_define = "
- type: todo
  display: Todo
  fields:
    - title: Title
    - author: String
";

    let file = EntryDefines {
        entries: serde_yaml::from_str(todo_define).unwrap(),
    };

    fs::write(define, serde_yaml::to_string(&file)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::action::entry_paths::EntryPaths;
    use crate::action::entry_usecases::sync_in_path;
    use crate::{process_cmd, Command, Init, Opts, SubCommand};
    use quake_core::entry::entry_file::EntryFile;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn should_throw_not_exist_cmds() {
        let command = Command {
            config: ".quake.yaml".to_string(),
            input: "story.dddd".to_string(),
            editor: "".to_string(),
        };
        let expected = process_cmd(Opts {
            cmd: SubCommand::Cmd(command),
        })
        .expect_err("");

        let error_msg = "QuakeError(\"unknown entry action: ActionDefine { object: \\\"story\\\", action: \\\"dddd\\\", text: \\\"\\\", parameters: [] }\")";
        assert_eq!(format!("{:?}", expected), error_msg);
    }

    #[test]
    fn should_create_test_entry() {
        let test_dir = "test_dir";

        let conf_dir = PathBuf::from("_fixtures")
            .join("configs")
            .join(".quake.yaml");

        let command = Command {
            config: format!("{:}", conf_dir.display()),
            input: "water.add: samples".to_string(),
            editor: "".to_string(),
        };

        process_cmd(Opts {
            cmd: SubCommand::Init(Init {
                path: test_dir.to_string(),
            }),
        })
        .unwrap();
        process_cmd(Opts {
            cmd: SubCommand::Cmd(command),
        })
        .unwrap();

        let test_path = PathBuf::from("test_dir");
        let paths = EntryPaths::init(&format!("{:}", test_path.display()), &"water".to_string());

        let content = fs::read_to_string(paths.base.join("0001-samples.md")).unwrap();
        let file = EntryFile::from(content.as_str(), 1).unwrap();

        let title = file.field("title");
        assert_eq!(title.unwrap(), "samples");

        fs::remove_dir_all(test_dir).unwrap();
    }

    #[ignore]
    #[test]
    fn placeholder() {
        let paths = EntryPaths::init(&"_fixtures".to_string(), &"notes".to_string());
        sync_in_path(&paths).unwrap();

        let paths = EntryPaths::init(&"_fixtures".to_string(), &"blog".to_string());
        sync_in_path(&paths).unwrap();
    }

    #[ignore]
    #[test]
    fn sync_todo() {
        let paths = EntryPaths::init(&"_fixtures".to_string(), &"microsoft_todo".to_string());
        sync_in_path(&paths).unwrap();
    }
}
