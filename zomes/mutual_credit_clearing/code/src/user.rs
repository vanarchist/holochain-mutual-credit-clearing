use std::convert::TryFrom;
use hdk::{
  AGENT_ADDRESS,
  entry_definition::ValidatingEntryType,
  error::ZomeApiResult,
  holochain_persistence_api::{
    cas::content::{AddressableContent, Address},
  },
  holochain_json_api::{
    error::JsonError, json::{JsonString, default_to_json},
  },
  holochain_core_types::{
    dna::entry_types::Sharing,
    validation::EntryValidationData,
    entry::Entry,
    link::LinkMatch,
  }
};

use serde::Serialize;
use std::fmt::Debug;

const USER_ANCHOR: &str = "user_anchor";
const USER_ANCHOR_ENTRY: &str = "users";
const USER_ENTRY_TYPE_NAME: &str = "user";
const USER_REGISTRATION_LINK: &str = "user_registration";
const USER_NAME_MAX_LENGTH: usize = 50;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct User {
  pub agent: Address,
  pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse<T> {
  pub entry: T,
  pub address: Address
}

impl<T: Into<JsonString> + Debug + Serialize> From<GetResponse<T>> for JsonString {
  fn from(u: GetResponse<T>) -> JsonString {
    default_to_json(u)
  }
} 

pub fn handle_create_user(name: String) -> ZomeApiResult<Address> {

  // Uses anchor pattern for now, should switch to 
  // something less prone to DHT hot-spots
  let user = User { 
    agent: AGENT_ADDRESS.to_string().into(),
    name,
  };
    
  let entry = Entry::App(
    USER_ENTRY_TYPE_NAME.into(),
    user.into(),
  );
    
  let user_address = hdk::commit_entry(&entry)?;
    
  let anchor_entry = Entry::App(
    USER_ANCHOR.into(),
    USER_ANCHOR_ENTRY.into(),
  );
  
  let anchor_address = hdk::commit_entry(&anchor_entry)?;
    
  hdk::link_entries(
    &anchor_address,
    &user_address,
    USER_REGISTRATION_LINK, 
    ""
  )?;
    
  Ok(user_address)
}

pub fn handle_get_users() -> ZomeApiResult<Vec<GetResponse<User>>> {

  let anchor_address = Entry::App(
    USER_ANCHOR.into(),
    USER_ANCHOR_ENTRY.into()
  ).address();
    
  Ok(
    hdk::utils::get_links_and_load_type(
      &anchor_address, 
      LinkMatch::Exactly(USER_REGISTRATION_LINK), // the link type to match
      LinkMatch::Any
    )?.into_iter().map(|user: User| {
      let address = Entry::App(USER_ENTRY_TYPE_NAME.into(), user.clone().into()).address();
      GetResponse{entry: user, address}
    }).collect()
  )
}

pub fn user_def() -> ValidatingEntryType {
  entry!(
    name: USER_ENTRY_TYPE_NAME,
    description: "Represents an agent registered on the network",
    sharing: Sharing::Public, 
    validation_package: || {
      hdk::ValidationPackageDefinition::Entry
    },
    validation: | validation_data: hdk::EntryValidationData<User>| {
      match validation_data {
        // only match if the entry is being created (not modified or deleted)
        EntryValidationData::Create{ entry, validation_data } => {
          let user = User::from(entry);
          let mut local_chain = validation_data.package.source_chain_entries;
          
          // TODO: not working
          if local_chain.is_some() {
            validate_user_not_registered(local_chain.unwrap(), &user.agent)?;
          }
          
          validate_user_name(&user.name)?;
          
          // user can only register themselves
          //if !validation_data.sources().contains(&user.agent) {
          //  return Err("Cannot register a user from another agent".into());
          //}
          
          // ** LOOK INTO THIS **
          // Sometimes the validating entry is already in the chain when validation runs,
          // To make our state reduction work correctly this must be removed
          //local_chain.remove_item(&Entry::App("user".into() , user.clone().into()));
          // TODO: finish this
          
          
          // otherwise no problems
          Ok(())
        },
        _ => {
          Err("Cannot modify or delete user".into())
        }
      }
    },
    links: [
      to!(
        // TODO: update these strings with consts
        "transaction",
        link_type: "from_user",
        validation_package: || { hdk::ValidationPackageDefinition::Entry },
        validation: | _validation_data: hdk::LinkValidationData| {
          Ok(())
        }
      )
    ]
  )
}

pub fn anchor_def() -> ValidatingEntryType {
  entry!(
    name: USER_ANCHOR,
    description: "Central known location to link from",
    sharing: Sharing::Public, 
    validation_package: || { hdk::ValidationPackageDefinition::Entry },
    validation: | _validation_data: hdk::EntryValidationData<String>| {
      Ok(())
    },
    links: [
      to!(
        USER_ENTRY_TYPE_NAME, 
        link_type: USER_REGISTRATION_LINK,
        validation_package: || {
          hdk::ValidationPackageDefinition::Entry
        },
        validation: | _validation_data: hdk::LinkValidationData| {
          Ok(())
        }
      )
    ]
  )
}

// Check if user name string is valid
pub fn validate_user_name(name: &String) -> Result<(), String> {
  // user name string length must be limited
  if name.len() > USER_NAME_MAX_LENGTH {
    return Err("User name string too long".into());
  }
  // user name string length should not be empty
  else if name.is_empty() {
    return Err("User name string cannot be empty".into());
  }
  else {
    Ok(())
  }
}

// check if user has already been registered for agent
pub fn validate_user_not_registered(local_chain: Vec<Entry>, agent_address: &Address) -> Result<(), String> {
  let found =
  local_chain
    .iter()
    .filter_map(|entry| {
      if let Entry::App(entry_type, entry_data) = entry {
        if entry_type.to_string() == USER_ENTRY_TYPE_NAME {
            Some(User::try_from(entry_data.clone()).unwrap())
        } else {
            None
        }
      } else {
          None
      }
    })
    .any(|user| user.agent == agent_address.to_owned());
    
  if found {
    Err("Agent can only register once".into())
  }
  else {
    Ok(())
  }
}

#[cfg(test)]
pub mod tests {

  use super::*;
  
  use hdk::{
    holochain_core_types::{entry::test_entry, entry::Entry},
    holochain_persistence_api::{
      cas::content::{Address},
    },
  };

  #[test]
  fn validate_user_name_empty() {
    assert!(validate_user_name(&"".to_string()).is_err());
  }
  
  #[test]
  fn validate_user_name_too_long() {
    assert!(validate_user_name(&"a".repeat(USER_NAME_MAX_LENGTH+1)).is_err());
  }
  
  #[test]
  fn validate_user_not_registered_yes() {
    let entry = test_entry();
    let addr = Address::from("test_addr");
    let user = User { 
      agent: addr.clone(),
      name: "Nick".to_string()
    };
    let entry = Entry::App(
      USER_ENTRY_TYPE_NAME.into(),
      user.into(),
    );
    let mut registered_users = Vec::new();
    registered_users.push(entry);
    assert!(validate_user_not_registered(registered_users, &addr).is_err())
  }
  
  #[test]
  fn validate_user_not_registered_no() {
    let entry = test_entry();
    let addr1 = Address::from("test_addr1");
    let addr2 = Address::from("test_addr2");
    let user = User { 
      agent: addr1,
      name: "Nick".to_string()
    };
    let entry = Entry::App(
      USER_ENTRY_TYPE_NAME.into(),
      user.into(),
    );
    let mut registered_users = Vec::new();
    registered_users.push(entry);
    assert!(validate_user_not_registered(registered_users, &addr2).is_ok())
  }

}


