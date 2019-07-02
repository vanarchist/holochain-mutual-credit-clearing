module.exports = scenario => {
  scenario("User registration", async (s, t, { amy, brad }) => {
    // user list empty to start
    const users = await amy.callSync("mutual_credit_clearing", "get_users", {});
    t.equal(users.Ok.length, 0, "User list empty to start")
  
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
  })
}
