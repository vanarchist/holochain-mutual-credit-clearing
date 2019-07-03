# holochain-mutual-credit-clearing
A minimal mutual credit clearing currency implementation on holochain

## Overview
This project was inspired by Thomas Greco's book, *The End of Money and the Future of Civilization* and Holochain DevCamp. The focus of the DevCamp was on developing a game with the provided framework. However it was also mentioned that many applications have rules like games including currencies. Since I am more interested in currencies than games I decided to try and build one outside of the DevCamp provided game framework but using similar design patterns. The table below shows a credit clearing ledger with four agents transacting and was taken from *The End of Money and the Future of Civilization*. 

![credit clearing](https://i.imgur.com/52Beln5.jpg[/img)

Holochain is agent-centric and eventually consistent which means viewing the ledger like what is shown in the table above is not quite the right way to look at things. That kind of global state implies agreement of every agent on the balance of every agent and transaction ordering which would require consensus. Instead, each agent (Amy, Brad, Carl, and Doris in the table) will have their own perspective on the balance of other agents.

## Design
The architecture is based on a countersigning pattern. More details and diagrams are forthcoming.

## Running
### Back End
Start the holochain node for the application by running ```holochain -c ./conductor-config.toml``` from the project root directory.
### Front End
Communication with the holochain node can be made with the appropriate JSON RPC calls from the tools of your choosing. As an example, a command line interface is included in the project. You can run the command line interface by changing to the ```examples/cli-example/``` directory and issuing the command ```cargo run http://localhost:8888 instance1```.
#### Example Command Line Interface
After starting the interface you will be greeted with the following prompt.
```
######################################################################
CLI example for holochain mutual credit clearing library.
Enter "help" for a list of commands.
Press Ctrl-D or enter "quit" to exit.
######################################################################


>
```
In order to transact with others on the credit clearing network, you need to register yourself as a user. Enter the command ```register``` with the user name of your choosing such as ```register Amy```. Next, we'll need to create another user to transact with. Open a new terminal and run the command ```cargo run http://localhost:8888 instance2``` from the ```examples/cli-example/``` directory to connect to another instance. At the prompt register another user. Here we'll do ```register Brad```.

To see a list of registered users, enter the command ```get_users```. You should see the following output:
```
Registered users: 

[QmSbkMsSMAgHEsdyT2pyAGsw4xh79nEfnB3WqERyMNEWn4] : { Agent: "HcScjcgKqXC5pmfvka9DmtEJwVr548yd86UPtJGGoue9ynuikuRTN7oE5zcjgbi", Name: "Amy" }
[QmUZruhfdzXE5zmixSBwyV7qV6Bsv9TVrNeGzfgondanZQ] : { Agent: "HcScidPSdAT43q9qirJwt5rHJYjjsvougV3jgSBwdJujszw3bBu5Mktr74Rgnea", Name: "Brad" }
```
The hash in [] is the user entry hash which is what we'll use when making transactions. 

## Tests
### Unit Tests
Unit tests are written in rust using the cargo test framework. These tests are particularly important for ensuring correctness of the validation logic. The unit tests can be run by changing to the ```zomes/mutual_credit_clearing/code/``` directory and running ```cargo test```.

### End-To-End Tests
End-to-end tests are written in javascript using the holochain diorama test framework. These tests are important for ensuring correctness of the application on the level of actual interacting users. The end-to-end tests can be run by issuing the command ```
hc test``` from the project root directory.
