use halfbrown::HashMap;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::{env, fs::File, os::unix::net::UnixStream, path::Path, process::Command};
use structs::{Config, Monitor, Workspace};

mod structs;

lazy_static! {
    static ref TRIM_START_REGEX: Regex = Regex::new(r"(?m)\s+\(").unwrap();
    static ref TRIM_END_REGEX: Regex = Regex::new(r"(?m)\)\s+").unwrap();
}

fn main() {
    let config = load_config();
    let mut workspace = load_workspaces();
    let mut monitors = load_monitors();

    print_yuck(&config, &workspace, &monitors);
    handle_event(&config, &mut workspace, &mut monitors);
}

fn print_yuck(
    config: &Config,
    workspaces: &HashMap<u8, Workspace>,
    monitors: &HashMap<String, Monitor>,
) {
    let mut result = String::new();

    for (id, icon) in config.workspaces.iter().sorted() {
        let mut state = "unoccupied".to_owned();

        match workspaces.get(id) {
            Some(workspace) => {
                let monitor_name = workspace.monitor_name.clone().unwrap();
                let monitor = monitors.get(&monitor_name).unwrap();

                if monitor.active_workspace.id == workspace.id {
                    state = format!("focused_{}", monitor_name.replace("-", "_"));
                } else {
                    state = "occupied".to_owned();
                }
            }
            None => {}
        }

        let body = config
            .body_template
            .replace("{id}", &id.to_string())
            .replace("{state}", &state)
            .replace("{icon}", icon);

        result.push_str(&body);
    }

    let result = config.template.replace("{body}", &result);

    println!(
        "{}",
        TRIM_END_REGEX.replace_all(TRIM_START_REGEX.replace_all(&result, "(").as_ref(), ")")
    );
}

fn handle_event(
    config: &Config,
    workspaces: &mut HashMap<u8, Workspace>,
    monitors: &mut HashMap<String, Monitor>,
) {
    let signature =
        env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("hyprland instance is not found");
    let socket_path_str = format!("/tmp/hypr/{signature}/.socket2.sock");
    let socket_path = Path::new(&socket_path_str);
    let stream = UnixStream::connect(&socket_path).expect("event socket is closed");
    let mut conn = BufReader::new(stream);

    loop {
        let mut buf = String::new();

        conn.read_line(&mut buf).expect("failed to read event");

        let mut split = buf.trim().split(">>");
        let event = split.next().unwrap();
        let value = split.next().unwrap();

        match event {
            "monitoradded" | "monitorremoved" => {
                *workspaces = load_workspaces();
                *monitors = load_monitors();
            }
            "focusedmon" => {
                split = value.split(",");
                let monitor_name = split.next().unwrap();
                let workspace_id = split.next().unwrap().parse::<u8>().unwrap();

                for (k, v) in monitors.iter_mut() {
                    if k == monitor_name {
                        v.focused = true;
                        v.active_workspace.id = workspace_id;
                    } else {
                        v.focused = false
                    }
                }
            }
            "moveworkspace" => {
                split = value.split(",");
                let workspace_id = split.next().unwrap().parse::<u8>().unwrap();
                let monitor_name = split.next().unwrap();

                monitors.get_mut(monitor_name).unwrap().active_workspace.id = workspace_id;
                workspaces.get_mut(&workspace_id).unwrap().monitor_name =
                    Some(monitor_name.to_owned());
            }
            "createworkspace" => {
                let workspace_id = value.parse::<u8>().unwrap();
                let focusing_monitor = monitors.values().find_or_first(|el| el.focused).unwrap();

                workspaces.insert(
                    workspace_id,
                    Workspace {
                        id: workspace_id,
                        monitor_name: Some(focusing_monitor.name.clone()),
                    },
                );
            }
            "workspace" => {
                let workspace_id = value.parse::<u8>().unwrap();
                let focusing_monitor = monitors.values().find_or_first(|el| el.focused).unwrap();
                let workspace = match workspaces.get(&workspace_id) {
                    Some(res) => res,
                    None => {
                        workspaces.insert(
                            workspace_id,
                            Workspace {
                                id: workspace_id,
                                monitor_name: Some(focusing_monitor.name.clone()),
                            },
                        );
                        workspaces.get(&workspace_id).unwrap()
                    }
                };
                let monitor_name = workspace
                    .monitor_name
                    .clone()
                    .unwrap_or(focusing_monitor.name.clone());

                monitors.get_mut(&monitor_name).unwrap().active_workspace.id = workspace_id;
            }
            "destroyworkspace" => {
                let workspace_id = value.parse::<u8>().unwrap();

                workspaces.remove(&workspace_id);
            }
            _ => continue,
        }

        print_yuck(config, workspaces, monitors)
    }
}

fn load_config() -> Config {
    let mut dir = env::current_exe().unwrap();

    dir.pop();
    dir.push("config.yaml");

    let f = File::open(dir).expect("config.json is not found");

    serde_yaml::from_reader(f).expect("failed to parse yaml")
}

fn execute_command(params: &[&'static str]) -> Vec<u8> {
    Command::new("hyprctl")
        .args([params, &["-j"]].concat())
        .output()
        .expect("failed to execute command")
        .stdout
}

fn load_workspaces() -> HashMap<u8, Workspace> {
    let result = execute_command(&["workspaces"]);
    let workspaces: Vec<Workspace> = serde_json::from_slice(&result).expect("failed to parse json");
    let mut workspaces_map: HashMap<u8, Workspace> = HashMap::new();

    for workspace in workspaces.iter() {
        workspaces_map.insert(workspace.id, workspace.to_owned());
    }

    workspaces_map
}

fn load_monitors() -> HashMap<String, Monitor> {
    let result = execute_command(&["monitors"]);
    let monitors: Vec<Monitor> = serde_json::from_slice(&result).expect("failed to parse json");
    let mut monitors_map: HashMap<String, Monitor> = HashMap::new();

    for monitor in monitors.iter() {
        monitors_map.insert(monitor.name.to_owned(), monitor.to_owned());
    }

    monitors_map
}
