module.exports = scenario => {
  scenario("User registration", async (s, t, { alice, bob }) => {
    // user list empty to start
    const users = await alice.call('mutual_credit_clearing', 'get_users');
    t.equal(users.length, 0, "User list empty to start")
  })
}
