## Weeks:
* 1 => Warm-up, In this Week i read about the theme of the project Rust and sipa Dns seeder: https://github.com/sipa/bitcoin-seeder ✅

* 2 => Connect to the Bitcoin Network, in this week i implement a basic client, with handshake, get addr and ping pong protocols, one of the issues that i have was with  getinv message when i dint respond the peer close the connection ✅

* 3 => Give It Ears,  Building a Command-Line Interface, in this week i create a very basic clap cli to change some default values of my code like the connection with the sipa code can be changed by another node ip... ✅

* 4 => Database,  In this week i implement a basic way to armazenate this peers in json file, with a status of what i alredy fo with this json and if it still alives or i alredy try a connection to him... ✅

* 5 => Multhread ✅ => In this week using tokio i implement a multhread connection with a comand in my rpc called "crawl" and with that i can connect with some of my list of peers implementede previously on my database

* 6 => Log system ✅ => In this week following the proposal i create  enum of states of log that i want and it returns more or less information about my conections, and i can call log() in my methods with the level trace and the thing that i want to log and it will be returned like a log in my cli if put in my input the same or mor trace of it.

* 7 => dns seeder by ip ❗✅ alredy has the comand and the logic but its not completely working and the door still close after all, needs debug.

* 8 => RPC start, stop .... ✅