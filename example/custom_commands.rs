// Note: this example is outdated! I'm too lazy to rewrite it right now.

/**
* custom_commands
* This sample plugin allows authorized users to create custom commands with !new-cmd <cmd-name> <content>.
* Other users can use these commands with !<cmd-name>, and it will display <content>
* Commands can then be removed with !del-cmd <cmd-name>.
*/

use std::{collections::HashMap, future::Future, process::Output};

use omegga::{OmeggaWrapper, Omegga, serde_json::{self, Value, json}, smol, rpc::RpcMessage};

struct CustomCommand {
    name: String,
    content: String
}

fn main() {
    let mut o = OmeggaWrapper::new();
    
    let stream = o.stream.clone();
    let omegga = o.clone_inner();
    
    smol::spawn(async move {
        let mut commands = vec![CustomCommand { name: "ping".into(), content: "Pong, test custom command".into() }];
        let mut authed_users = vec![String::from("x")];
        
        let blocked_commands = ["new-cmd", "del-cmd", "ls-cmds"];
        
        while let Ok(message) = stream.recv().await {
            match message {
                RpcMessage::Notification { method, params, .. } => {
                    match method.as_str() {
                        "chat" => {
                            let params = params.unwrap();
                            let (user, content) = (&params[0].as_str().unwrap(), &params[1].as_str().unwrap());
                            
                            for command in commands.iter() {
                                if content.to_lowercase().starts_with(format!("!{}", command.name).as_str()) {
                                    Omegga::notify("broadcast", json!(command.content));
                                }
                            }
                        },
                        "chatcmd:new-cmd" => {
                            let params = params.unwrap();
                            let params_array = params.as_array().unwrap();
                            let (user, name, args) = (
                                params_array[0].as_str().unwrap(),
                                params_array[1].as_str().unwrap(),
                                params_array[2..]
                                .iter()
                                .map(|v| v.as_str().unwrap())
                                .collect::<Vec<&str>>());
                                
                                // reject unauthed users
                                if !authed_users.contains(&user.into()) {
                                    continue;
                                }
                                
                                // reject plugin-used commands
                                if blocked_commands.contains(&name.to_lowercase().as_str()) {
                                    continue;
                                }
                                
                                // check if the command exists
                                if commands.iter().any(|c| c.name == name.to_lowercase()) {
                                    Omegga::notify("broadcast", json!("A custom command already exists with that name."));
                                    continue;
                                }
                                
                                commands.push(CustomCommand { name: name.to_lowercase().into(), content: args.join(" ") });
                                Omegga::notify("broadcast", json!("OK, created that new custom command."));
                            },
                            "chatcmd:del-cmd" => {
                                let params = params.unwrap();
                                let (user, name) = (params[0].as_str().unwrap(), params[1].as_str().unwrap());
                                
                                // reject unauthed users
                                if !authed_users.contains(&user.into()) {
                                    continue;
                                }
                                
                                // remove from commands
                                let before_len = commands.len();
                                commands.retain(|c| c.name != name.to_lowercase());
                                
                                if before_len > commands.len() {
                                    Omegga::notify("broadcast", json!("Removed that command."));
                                } else {
                                    Omegga::notify("broadcast", json!("No command by that name found."));
                                }
                            },
                            "chatcmd:ls-cmds" => { // display a list of registered commands
                                Omegga::notify("broadcast", json!("<b>List of Custom Commands</>"));
                                for chunk in commands.chunks(5) {
                                    let names = chunk
                                    .iter()
                                    .map(|c| format!("!{}", c.name))
                                    .collect::<Vec<String>>();
                                    
                                    Omegga::notify("broadcast", json!(names.join(", ")));
                                }
                            },
                            "chatcmd:auth" => { // authorize another user to add commands
                                let params = params.unwrap();
                                let (user, target) = (params[0].as_str().unwrap(), params[1].as_str().unwrap());
                                
                                // reject unauthed users
                                if !authed_users.contains(&user.into()) {
                                    continue;
                                }
                                
                                // error if target is already authed
                                if authed_users.contains(&target.into()) {
                                    Omegga::notify("broadcast", json!("That user is already authed!"));
                                    continue;
                                }
                                
                                // auth user
                                authed_users.push(String::from(target));
                                Omegga::notify("broadcast", json!("Authorized that user."));
                            },
                            _ => ()
                        }
                    },
                    RpcMessage::Request { id, method, params, .. } => {
                        match method.as_str() {
                            "init" => {
                                // send a blank response to let omegga know we work
                                Omegga::respond(id, Ok(json!({})));
                                
                                // print out some text (todo: a more abstracted api)
                                Omegga::notify("log", json!("Hello from omegga-rs!"));
                            },
                            "stop" => {
                                Omegga::notify("log", json!("Server requested stop..."));
                                Omegga::respond(id, Ok(json!(0)));
                            },
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }
        }).detach();
        
        o.start();
    }
    