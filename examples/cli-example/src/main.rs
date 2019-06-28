use std::io;
use std::iter::repeat;
use linefeed::{Interface, ReadResult};

static COMMANDS: &[(&str, &str)] = &[
  ("help", "Displays this the help page"),
  ("exit", "Exit this CLI."),
];

fn main() -> io::Result<()> {

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
