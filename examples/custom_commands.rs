/**
 * custom_commands
 * This sample plugin allows users to create custom commands with !new-cmd <cmd-name> <content>.
 * Other users can use these commands with !<cmd-name>, and it will display <content>
 * Commands can then be removed with !del-cmd <cmd-name>, and all can be viewed with !ls-cmds.
 */
use std::collections::HashMap;

use omegga::{events::Event, Omegga};

#[tokio::main]
async fn main() {
    let omegga = Omegga::new();
    let mut rx = omegga.spawn();

    let mut commands = HashMap::new();

    while let Some(event) = rx.recv().await {
        match event {
            Event::Init { id, .. } => {
                omegga.register_commands(id, &[]);
                omegga.log("Hello from omegga-rs!");
            }
            Event::Stop { id, .. } => omegga.write_response(id, None, None),

            Event::ChatCommand { command, args, .. } => match command.as_str() {
                "new-cmd" => {
                    let mut args = args.into_iter();
                    let name = args.next().unwrap();
                    let content = args.collect::<Vec<_>>().join(" ");

                    if let None = commands.insert(name.clone(), content) {
                        omegga.broadcast(format!("OK, created the custom command {}.", name));
                    } else {
                        omegga.broadcast(format!("OK, overwrote that existing custom command."));
                    }
                }
                "del-cmd" => {
                    let name = args.into_iter().next().unwrap();
                    if let Some(_) = commands.remove(&name) {
                        omegga.broadcast(format!("OK, removed the custom command {}.", name));
                    } else {
                        omegga.broadcast("That custom command didn't exist.");
                    }
                }
                "ls-cmds" => {
                    omegga.broadcast("<b>Custom Commands</>");
                    for chunk in commands.keys().collect::<Vec<&String>>().chunks(5) {
                        let names = chunk
                            .iter()
                            .map(|cmd| format!("<code>!{}</>", cmd))
                            .collect::<Vec<_>>()
                            .join(", ");
                        omegga.broadcast(names);
                    }
                }
                cmd => {
                    if let Some(content) = commands.get(cmd) {
                        omegga.broadcast(content);
                    }
                }
            },
            _ => (),
        }
    }
}
