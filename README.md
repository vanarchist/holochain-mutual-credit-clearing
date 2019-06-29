# holochain-mutual-credit-clearing
A minimal mutual credit clearing currency implementation on holochain

## Overview
This project was inspired by Thomas Greco's book, *The End of Money and the Future of Civilization* and Holochain DevCamp. The focus of the DevCamp was on developing a game with the provided framework. However it was also mentioned that many applications have rules like games including currencies. Since I am more interested in currencies than games I decided to try and build one outside of the DevCamp provided game framework but using similar design patterns. The table below shows a credit clearing ledger with four agents transacting and was taken from *The End of Money and the Future of Civilization*. 

![credit clearing](https://i.imgur.com/52Beln5.jpg[/img)

Holochain is agent-centric and eventually consistent which means viewing the ledger like what is shown in the table above is not quite the right way to look at things. That kind of global state implies agreement of every agent on the balance of every agent and transaction ordering which would require consensus. Instead, each agent (Amy, Brad, Carl, and Doris in the table) will have their own perspective on the balance of other agents.
