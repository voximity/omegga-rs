/**
 * custom_commands
 * This sample plugin allows users to create custom commands with !new-cmd <cmd-name> <content>.
 * Other users can use these commands with !<cmd-name>, and it will display <content>
 * Commands can then be removed with !del-cmd <cmd-name>, and all can be viewed with !ls-cmds.
 */

 use std::collections::HashMap;

 use omegga::{Omegga, rpc};
 
 #[tokio::main]
 async fn main() {
     let omegga = Omegga::new();
     let mut rx = omegga.spawn();
 
     let mut commands = HashMap::new();
 
     while let Some(message) = rx.recv().await {
         match message {
             rpc::Message::Request { method, id, .. } if method == "init" || method == "stop" => {
                 // just write anything so omegga knows we work
                 omegga.write_response(id, None, None);
 
                 if method == "init" {
                     omegga.log("Hello from omegga-rs!");
                 }
             }
             rpc::Message::Notification { method, params, .. } if method == "chatcmd:new-cmd" => {
                 // add a new command
                 let params = params.unwrap();
                 let mut params = params.as_array().unwrap().into_iter();
                 let _user = params.next().unwrap().as_str().unwrap();
                 let name = params.next().unwrap().as_str().unwrap();
                 let content = params.map(|v| v.as_str().unwrap()).collect::<Vec<&str>>();
 
                 if let None = commands.insert(String::from(name), content.join(" ")) {
                     omegga.broadcast(format!("OK, created the custom command {}.", name));
                 } else {
                     omegga.broadcast("OK, overwrite that existing custom command.");
                 }
             }
             rpc::Message::Notification { method, params, .. } if method == "chatcmd:del-cmd" => {
                 // delete a command
                 let params = params.unwrap();
                 let mut params = params.as_array().unwrap().into_iter();
                 let _user = params.next().unwrap().as_str().unwrap();
                 let name = params.next().unwrap().as_str().unwrap();
 
                 if let Some(_) = commands.remove(name) {
                     omegga.broadcast(format!("OK, removed the custom command {}.", name));
                 } else {
                     omegga.broadcast("That custom command didn't exist.");
                 }
             }
             rpc::Message::Notification { method, params, .. } if method == "chatcmd:ls-cmds" => {
                 // list all custom commands
                 let params = params.unwrap();
                 let mut params = params.as_array().unwrap().into_iter();
                 let _user = params.next().unwrap().as_str().unwrap();
 
                 omegga.broadcast("<b>Custom Commands</>");
                 for chunk in commands.keys().collect::<Vec<&String>>().chunks(5) {
                     let names = chunk.iter().map(|cmd| format!("<code>!{}</>", cmd)).collect::<Vec<_>>().join(", ");
                     omegga.broadcast(names);
                 }
             }
             rpc::Message::Notification { method, params, .. } if method == "chat" => {
                 let params = params.unwrap();
                 let (_user, content) = (&params[0].as_str().unwrap(), &params[1].as_str().unwrap());
 
                 if let Some((_, value)) = commands.iter().find(|(command, _)| *content == format!("!{}", command)) {
                     omegga.broadcast(value);
                 }
             }
             _ => (),
         }
     }
 }
 