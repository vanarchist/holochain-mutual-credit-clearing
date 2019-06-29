use std::io;
use std::iter::repeat;
use linefeed::{Interface, ReadResult};
use serde_json::json;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Cli {
  /// Url to connect to the running conductor HTTP port (e.g. http://localhost:3000)
  url: reqwest::Url,
  /// This is the instance ID in the conductor that is running the app on
  instance: String,
}

static COMMANDS: &[(&str, &str)] = &[
  ("register", "Register as a participant of the credit clearing network. Usage: register <name>"),
  ("get_users", "Get all registered users of the credit clearing network."),
  ("help", "Displays this the help page"),
  ("exit", "Exit this CLI."),
];

fn main() -> io::Result<()> {

  let cli = Cli::from_args();
  
  // calls to zome functions
  let register = holochain_call_generator(cli.url.clone(), 
                                          cli.instance.clone(), 
                                          "mutual_credit_clearing".into(), 
                                          "create_user".into());
  let get_users = holochain_call_generator(cli.url.clone(), 
                                           cli.instance.clone(), 
                                           "mutual_credit_clearing".into(), 
                                           "get_users".into());

  let interface = Interface::new("Holochain mutual credit clearing CLI")?;
  
  println!("");
  println!("");
  println!("{}", repeat('#').take(70).collect::<String>());
  println!("CLI example for holochain mutual credit clearing library.");
  println!("Enter \"help\" for a list of commands.");
  println!("Press Ctrl-D or enter \"quit\" to exit.");
  println!("{}", repeat('#').take(70).collect::<String>());
  println!("");
  println!("");
  
  interface.set_prompt("> ")?;
  
  while let ReadResult::Input(line) = interface.read_line()? {
    if !line.trim().is_empty() {
      interface.add_history_unique(line.clone());
    }

    let (cmd, args) = split_first_word(&line);
    
    let result: Result<(), String> = match cmd {
      "register" => {
        println!("registering user with name {:?}", args);
        let result = register(json!({"name": args}));
        println!("Create result: {:?}", result);
        Ok(())
      }
      "get_users" => {
        let result = get_users(json!({})).unwrap();
        println!("Registered users: \n");
        result.as_array().unwrap().iter().for_each(|r| {
          println!("[{}] : {{ Agent: {}, Name: {} }}", 
                   r["address"].as_str().unwrap(), 
                   r["entry"]["agent"], 
                   r["entry"]["name"]);
        });
        println!("\n");
        Ok(())
      }
      "help" => {
        println!("Holochain mutual credit clearing CLI commands:");
        println!();
        for &(cmd, help) in COMMANDS {
          println!("  {:15} - {}", cmd, help);
        }
        println!();
        Ok(())
      }
      "exit" => {
        println!("Bye!");
        break;
      }
      _ => {
        Err("Invalid command!".into())
      }
    };
    
    if let Err(e) = result {
      println!("Error: {}", e)
    }
  }  
  
  Ok(())
}

/**
 * Returns functions to make calls to a particular zome function on a url
 */
fn holochain_call_generator(
  url: reqwest::Url, 
  instance: String,
  zome: String,
  func: String,
) -> Box<Fn(serde_json::Value) -> Result<serde_json::Value, String>> {

  let client = reqwest::Client::new();

  let make_rpc_call = move |params| {
    json!({
      "jsonrpc": "2.0",
      "id": 0,
      "method": "call",
      "params": {
        "instance_id": instance,
        "zome": zome,
        "function": func,
        "args": params
      }
    })
  };

  Box::new(move |params| {
    let call_result: serde_json::Value = client.post(url.clone())
      .json(&make_rpc_call(params))
      .send().map_err(|e| e.to_string())?
      .json()
      .map(|r: serde_json::Value| {
        r["result"].clone()
      })
      .map(|s| serde_json::from_str(
        s.as_str().expect(&format!("Holochain did not return a string result: {}", s))
      ).expect(&format!("Holochain did not return a valid stringified JSON result: {}", s)))
      .map_err(|e| e.to_string())?;

    // deal with the json encoded holochain error responses
    if let Some(inner_result) = call_result.get("Ok") {
      Ok(inner_result.clone())
    } else {
      Err(call_result["Err"].to_string())
    }
  })
}

/*===============================
=            Helpers            =
===============================*/

fn split_first_word(s: &str) -> (&str, &str) {
  let s = s.trim();

  match s.find(|ch: char| ch.is_whitespace()) {
    Some(pos) => (&s[..pos], s[pos..].trim_start()),
    None => (s, "")
  }
}
