use hdk::{
    AGENT_ADDRESS,
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};
use hdk::holochain_core_types::{
    cas::content::Address,
    entry::Entry,
    dna::entry_types::Sharing,
    error::HolochainError,
    json::{JsonString, default_to_json},
    validation::EntryValidationData,
    cas::content::AddressableContent,
    link::LinkMatch,
};
use serde::Serialize;
use std::fmt::Debug;

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
    "user".into(),
    user.into(),
  );
    
  let user_address = hdk::commit_entry(&entry)?;
    
  let anchor_entry = Entry::App(
    "anchor".into(),
    "users".into(),
  );
  
  let anchor_address = hdk::commit_entry(&anchor_entry)?;
    
  hdk::link_entries(
    &anchor_address,
    &user_address,
    "user_registration", 
    ""
  )?;
    
  Ok(user_address)
}

pub fn handle_get_users() -> ZomeApiResult<Vec<GetResponse<User>>> {

  let anchor_address = Entry::App(
    "anchor".into(),
    "users".into()
  ).address();
    
  Ok(
    hdk::utils::get_links_and_load_type(
      &anchor_address, 
      LinkMatch::Exactly("user_registration"), // the link type to match
      LinkMatch::Any
    )?.into_iter().map(|user: User| {
      let address = Entry::App("user".into(), user.clone().into()).address();
      GetResponse{entry: user, address}
    }).collect()
  )
}

pub fn user_def() -> ValidatingEntryType {
  entry!(
    name: "user",
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
          
          // user name string length must be limited
          if user.name.len() > 50 {
            return Err("User name string too long".into());
          }
          
          // user can only register themself
          if !validation_data.sources().contains(&user.agent) {
            return Err("Cannot register a user from another agent".into());
          }
          
          // user can only register once
          //let mut local_chain = validation_data.package.source_chain_entries
          //  .ok_or("Could not retrieve source chain")?;
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
    links: []
  )
}

pub fn anchor_def() -> ValidatingEntryType {
  entry!(
    name: "anchor",
    description: "Central known location to link from",
    sharing: Sharing::Public, 
    validation_package: || { hdk::ValidationPackageDefinition::Entry },
    validation: | _validation_data: hdk::EntryValidationData<String>| {
      Ok(())
    },
    links: [
      to!(
        "user", 
        link_type: "user_registration",
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


