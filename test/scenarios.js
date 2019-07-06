module.exports = scenario => {
  scenario("User registration", async (s, t, { amy, brad }) => {
    // user list empty to start
    const users = await amy.callSync("mutual_credit_clearing", "get_users", {});
    t.equal(users.Ok.length, 0, "User list empty to start")
    
    // amy does not get user addr prior to registration
    const amy_no_usr = await amy.callSync("mutual_credit_clearing", "get_my_user", {});
    t.equal(amy_no_usr.Ok, undefined, "Amy can't get user address because not registered")
  
    // register amy as a user
    const amy_addr = await amy.callSync("mutual_credit_clearing", "create_user", {name: "Amy"})
    t.equal(amy_addr.Ok.length, 46, "Amy was registered successfully")
    
    // check that amy sees herself as registered user
    const registered = await amy.callSync("mutual_credit_clearing", "get_users", {});
    t.equal(registered.Ok.length, 1, "Amy sees number of registered users correct")
    t.deepEqual(registered.Ok,
                [{
                  entry: {
                    name: "Amy",
                    agent: amy.agentId
                  },
                  address: registered.Ok[0].address
                }],
                "Amy sees herself in the registered users"
               )
    
    // check that brad sees amy as registered user
    const brad_get = await brad.callSync("mutual_credit_clearing", "get_users", {});
    t.equal(brad_get.Ok.length, 1, "Brad sees number of registered users correct")
    t.deepEqual(brad_get.Ok,
                [{
                  entry: {
                    name: "Amy",
                    agent: amy.agentId
                  },
                  address: brad_get.Ok[0].address
                }],
                "Brad sees Amy in the registered users"
               )
    
    // Amy tries registering again
    const amy_dup = await amy.callSync("mutual_credit_clearing", "create_user", {name: "Amy"})
    t.equal(amy_dup.Ok, undefined, "Amy tries to register again")
    
    // Brad tries registering without specifing a user name
    const brad_empty = await brad.callSync("mutual_credit_clearing", "create_user", {name:""})
    t.equal(brad_empty.Ok, undefined, "Brad tries registering with empty string")
    
    // Brad tries registering with long user name
    const brad_long = await brad.callSync("mutual_credit_clearing", "create_user", {name:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"})
    t.equal(brad_long.Ok, undefined, "Brad tries registering with long name")     
    
    // register brad as a user
    const brad_addr = await brad.callSync("mutual_credit_clearing", "create_user", {name: "Brad"})
    t.equal(brad_addr.Ok.length, 46, "Brad was registered successfully")
    
    // check that amy sees herself and brad as registered users
    const registered_amy = await amy.callSync("mutual_credit_clearing", "get_users", {});
    t.equal(registered_amy.Ok.length, 2, "Amy sees two users")
    t.deepEqual(registered_amy.Ok,
                [{
                  entry: {
                    name: "Amy",
                    agent: amy.agentId
                  },
                  address: registered_amy.Ok[0].address
                  },
                  {
                  entry: {
                    name: "Brad",
                    agent: brad.agentId
                  },
                  address: registered_amy.Ok[1].address
                  }
                ],
                "Amy sees herself and Brad"
               )
  
    // check that brad can get his user address
    const brad_usr_addr = await brad.callSync("mutual_credit_clearing", "get_my_user", {});
    t.equal(brad_usr_addr.Ok.length, 46, "Returned brad's address")
    t.equal(brad_usr_addr.Ok, brad_addr.Ok, "Brad's returned address is correct")
  
    // check that amy can get her user address
    const amy_usr_addr = await amy.callSync("mutual_credit_clearing", "get_my_user", {});
    t.equal(amy_usr_addr.Ok.length, 46, "Returned amy's address")
    t.equal(amy_usr_addr.Ok, amy_addr.Ok, "Amy's returned address is correct")
  
    
  })
}
