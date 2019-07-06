//
// For this minimal implementation, a holochain agent id is tied to a user.
// The relationship is one-to-one, you can only have one user per agent. There
// is no verification that each agent is actually a unique person. So a person
// could still create multiple users by running multiple agents. 
// 

use std::convert::TryFrom;
use hdk::{
  AGENT_ADDRESS,
  entry_definition::ValidatingEntryType,
  error::{ZomeApiResult, ZomeApiError},
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

// get user address associated with this agentid if it exists
pub fn get_my_user() -> ZomeApiResult<Address> {
  let anchor_address = Entry::App(
    USER_ANCHOR.into(),
    USER_ANCHOR_ENTRY.into()
  ).address();
    
  let found =
    hdk::utils::get_links_and_load_type(
      &anchor_address, 
      LinkMatch::Exactly(USER_REGISTRATION_LINK), // the link type to match
      LinkMatch::Any
    )?.into_iter().map(|user: User| {
      user }).find(|user| user.agent == AGENT_ADDRESS.to_string().into());
  
  match found {
    Some(user) => {
      let address = Entry::App(USER_ENTRY_TYPE_NAME.into(), user.clone().into()).address();
      Ok(address)
    },
    None => {
      Err(ZomeApiError::HashNotFound)
    }
  }
}

pub fn user_def() -> ValidatingEntryType {
  entry!(
    name: USER_ENTRY_TYPE_NAME,
    description: "Represents an agent registered on the network",
    sharing: Sharing::Public, 
    validation_package: || {
      hdk::ValidationPackageDefinition::ChainFull
    },
    validation: | validation_data: hdk::EntryValidationData<User>| {
      match validation_data {
        // only match if the entry is being created (not modified or deleted)
        EntryValidationData::Create{ entry, validation_data } => {
    
          // need to find out what context this was called from
          // so we know what to expect on the local chain
          let lifecycle = validation_data.clone().lifecycle;
          
          let user = User::from(entry);
          
          // get full chain
          let local_chain = validation_data.clone().package.source_chain_entries
            .ok_or("Could not retrieve source chain")?;
          
          hdk::debug(format!("{:?}", local_chain))?;
          
          // validate that agent has not registered a user yet
          validate_user_not_registered(local_chain, &user.agent, lifecycle)?;
          
          // validate user name string
          validate_user_name(&user.name)?;
          
          // validate creating agent is specified agent
          // not sure this is really needed, how would this condition arise?
          if !validation_data.sources().contains(&user.agent) {
            return Err("Cannot register a user from another agent".into());
          }
          
          // if made it here, no problems
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
pub fn validate_user_not_registered(local_chain: Vec<Entry>, agent_address: &Address, lifecycle: hdk::EntryLifecycle) -> Result<(), String> {
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
  
  match lifecycle {
    hdk::EntryLifecycle::Chain => {
      if found {
        return Err("Agent can only register once".into());
      }
    }
    _ => {}
  }
          
  Ok(())
}


#[cfg(test)]
pub mod tests {

  use super::*;
  
  use hdk::{
    holochain_core_types::{entry::Entry},
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
    assert!(validate_user_not_registered(registered_users, &addr, 
            hdk::EntryLifecycle::Chain).is_err())
  }
  
  #[test]
  fn validate_user_not_registered_no() {
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
    assert!(validate_user_not_registered(registered_users, &addr2,
            hdk::EntryLifecycle::Chain).is_ok())
  }

}

