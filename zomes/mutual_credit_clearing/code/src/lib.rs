#![feature(try_from, vec_remove_item, proc_macro_hygiene)]
#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_json_derive;

extern crate hdk_proc_macros;
use hdk_proc_macros::zome;

use hdk::{
  error::ZomeApiResult,
  entry_definition::ValidatingEntryType,
  holochain_persistence_api::{
    cas::content::{Address},
  }
};

mod user;

use user::{User, GetResponse};

#[zome]
pub mod main {

  #[genesis]
  pub fn genesis() {
    Ok(())
  }

  /*=========================================
  =            Entry Definitions            =
  =========================================*/
  
  #[entry_def]
  fn user_entry_def() -> ValidatingEntryType {
    user::user_def()
  }
  
  #[entry_def]
  fn anchor_def() -> ValidatingEntryType {
    user::anchor_def()
  }
  
  /*======================================
  =            Zome functions            =
  ======================================*/
  
  #[zome_fn("hc_public")]
  fn create_user(name: String) -> ZomeApiResult<Address> {
    user::handle_create_user(name)
  }
  
  #[zome_fn("hc_public")]
  fn get_users() -> ZomeApiResult<Vec<GetResponse<User>>> {
    user::handle_get_users()
  }
  
}
