# Task 1: Warm-up

> *“Bitcoin is structured as a peer-to-peer network architecture on top of the internet.
> The term peer-to-peer, or P2P, means that the full nodes that participate in the network are peers to each other, that they can all perform the same functions, and that there are no ‘special’ nodes.
> The network nodes interconnect in a mesh network with a ‘flat’ topology.
> There is no server, no centralized service, and no hierarchy within the network.
> Nodes in a P2P network both provide and consume services at the same time.”*
> — A. Antonopoulos and D. Harding, *Mastering Bitcoin*[^1]

The predominant architecture on the internet is the classic **client-server** model.
In this design, servers and clients serve different purposes:
servers are typically more powerful, hold privileged data, and handle control over authentication and access.
Even in apps that allow client-to-client interaction (like messaging systems), those connections are mediated by a server.
Each client connects to the server, not directly to other clients.

In a **P2P architecture**, by contrast, all participants—called *nodes*—communicate directly with one another.
There is no privileged entity, which means nodes must be far more capable than typical clients.
One of their key responsibilities is to manage peer connections.
This includes *peer discovery*—learning about new nodes in the network.

As you’ll see in Task 2, the Bitcoin protocol includes mechanisms for this. Special messages are used to request and announce known peers. That way, even if your node is connected to just one peer, it can gradually learn about others and expand its network reach[^2]. But that raises an important question: *how does a new node connect to its first peer in the first place?*
This process is called **bootstrapping**[^3] into the network.

## Bitcoin DNS Seeders

While the Bitcoin protocol doesn’t require “special” nodes, in practice the network does rely on some nodes providing specialized services—for example, helping new nodes bootstrap into the network by advertising known peer addresses.

In Bitcoin’s early days, nodes discovered peers through online forums or IRC channels.
As the network grew, this became impractical.
The solution was **DNS seeders**.

A Bitcoin DNS Seeder is a special node that continually connects to the Bitcoin P2P network, collects peer addresses, attempts to connect to them, and maintains a database of known peers.
These addresses are categorized—for example, as “known” (seen in the network), “active” (successfully connected), or “inactive” (could not connect).
Additional statuses like “banned” can also be used for misbehaving peers.

The seeder also acts as a **DNS server**, answering queries with random sets of peer IP addresses in `A` records.
For example:

```bash
❯ dig seed.bitcoin.sipa.be

;; ANSWER SECTION:
seed.bitcoin.sipa.be.   2455    IN      A       57.129.38.163
seed.bitcoin.sipa.be.   2455    IN      A       139.177.179.5
... (other IPs) ...
```

The oldest known implementation, [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder), was created by Pieter Wuille and has been [hardcoded into Bitcoin Core](https://github.com/bitcoin/bitcoin/blob/dbc450c1b59b24421ba93f3e21faa8c673c0df4c/src/kernel/chainparams.cpp#L145) since 2011.
There are other implementations today—including some in Rust—but we encourage you not to look at them.
The goal is to understand the functionality and reimplement it yourself without bias.

---

## Your Tasks

### 1. Install and get acquainted with the Rust toolchain

- Study Chapters 1, 2, 3, and 7 of [The Rust Book](https://doc.rust-lang.org/stable/book/title-page.html).

### 2. Run the Bitcoin Seeder

Clone, build, and run Pieter Wuille’s [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder).
Your goal at this stage is not to understand every detail, but to get a feel for what the software does, how it behaves at runtime, and how it communicates with the outside world.

Use the following questions to guide your exploration:

#### 🧠 General Behavior

- What do you see in the terminal when the seeder starts? What kinds of activities does it log?
- Can you identify distinct phases or responsibilities (e.g., peer discovery, DNS service, logging)?
- What configuration options or runtime flags does the software support?

#### 🔍 Interactive Observation

- Use `dig` to query your local seeder (e.g., `dig @127.0.0.1 example.seed.local`).
  What kind of response do you get?
- Does the seeder log anything when a DNS query is received?
- What kind of information does it return in the DNS reply? Are the IPs random? Are they actual peers?

#### 🧠 Reflection

- Based on what you’ve observed, what services is the seeder providing?
- What signals is it giving you about its internal state—number of peers, health, errors?
- What do you *wish* it told you that it currently doesn’t?

### 3. Explore the Bitcoin Seeder’s Internal Structure

The Bitcoin Seeder is a multi-threaded application. Rather than a central loop controlling all behavior, it delegates work to specialized threads that run concurrently.

As you explore the codebase, try to answer the following:

#### 🧵 Concurrency and Threads

- What kinds of tasks run in parallel (e.g., DNS, peer probing, logging)?
- Where in the code are threads created, and why?
- Do threads appear to be long-lived or created on demand? How does the program manage their lifecycle?

#### 🗃️ State and Data Management

- What data structures are used to track peers? Are they easy to work with? Efficient?
- Is the database implementation something you’d reuse? If not, why?
  What would you do differently in Rust?

#### 🔍 Design Reflection

- Do any parts of the implementation feel overly complex or tightly coupled?
- What would you want to improve if you had to maintain or extend this codebase?

> Bonus: Try sketching a simple block diagram of how different parts of the system (DNS server, peer database, logger) interact.

---

### 4. Get acquainted with DNS

Later in the seminar, you’ll build your own DNS server—so now’s the time to get comfortable with the basics of how DNS works.

Focus on these questions as you explore:

#### 🌐 What is DNS?

- What is the basic role of the Domain Name System (DNS) in the internet stack?
- What happens when you type a domain name into a browser?

#### 🧾 What are DNS records?

- What are `A` and `AAAA` records? Why are they important for our project?
- What’s the difference between a DNS query and a DNS response?

#### 🧠 From Concept to Practice

- How do you perform a DNS query from the terminal (e.g., using `dig`)?
- Can you imagine how your software will handle multiple queries at once?

Start light:

- [Wikipedia: Domain Name System](https://en.wikipedia.org/wiki/Domain_Name_System)
- [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034) and [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035) — just skim the opening sections to get the high-level picture.

> You don’t need to master the full DNS protocol—just understand what it is, how queries and responses work, and why it's relevant to our seeder.

---

### 5. Get acquainted with the Bitcoin P2P protocol

Start with the [Bitcoin Wiki section on the protocol](https://en.bitcoin.it/wiki/Protocol_documentation#getaddr), especially how nodes exchange addresses (`getaddr`, `addr` messages).

---

## A Word of Warning

These first tasks are designed to get you set up and to think critically about what you are doing.
They seem quite easy and you might want to skip thinking—but don’t be fooled.
The difficulty will ramp up quickly.
This seminar is meant to be challenging and rewarding.
The more you think about what you are doing, the more you will learn.
Prepare to dive deep.

---

[^1]: A. Antonopoulos and D. Harding; *Mastering Bitcoin*
[^2]: Mainly IP addresses for TCP connections, though other transport layers exist.
[^3]: [Bootstrapping – Wikipedia](https://en.wikipedia.org/wiki/Bootstrapping#Etymology)