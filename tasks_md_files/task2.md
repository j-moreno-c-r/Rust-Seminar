# Task 2: Connect to the Bitcoin Network

> *“Before the first word is spoken, a protocol must be assumed.
> But who speaks first when no one is listening yet?”*

In the previous task, we explored how new nodes **bootstrap** into a peer-to-peer network.
Now it’s time to participate.
In this task, you'll build a simple Bitcoin client that connects to a node on the public network using the Bitcoin P2P protocol.

This protocol defines how peers exchange messages.
But unlike many internet protocols that rely on a **formal specification**, Bitcoin takes a different route:
**Bitcoin Core *is* the specification**.
There are efforts to document expected behaviors through [BIPs](https://github.com/bitcoin/bips), but the most reliable way to know what’s “correct” is to see what Bitcoin Core does.

That means studying code—not just specs.
To help you do that, we've linked directly to key parts of the [Bitcoin Core codebase](https://github.com/bitcoin/bitcoin) where relevant.

---

## A Word About the Protocol

Bitcoin's P2P layer is unencrypted and unauthenticated.
It assumes all data is public and operates over any byte stream (originally just TCP).
In practice, modern Bitcoin Core supports IPv6, Tor, and I2P as well.

All network messages follow the same high-level format[^1]:

- `magic` (4 bytes): identifies the network (mainnet, testnet, etc.)
- `command` (12 bytes): null-padded ASCII command string
- `payload_size` (4 bytes): length in bytes
- `checksum` (4 bytes): first 4 bytes of `SHA256(SHA256(payload))`
- `payload`: actual content

You’ll work with the following message types:

- `version`, `verack`: used during the handshake
- `ping`, `pong`: used to keep connections alive
- `getaddr`, `addr`: used to request and share peer information

Ignore the other message types for now[^2].

[^1]: [CMessageHeader class](https://github.com/bitcoin/bitcoin/blob/dfb7d58108daf3728f69292b9e6dba437bb79cc7/src/protocol.h#L28)
[^2]: See [`NetMsgType`](https://github.com/bitcoin/bitcoin/blob/dfb7d58108daf3728f69292b9e6dba437bb79cc7/src/protocol.h#L60)

---

## Your Task: Write a Basic Bitcoin Client

You’re going to write a small client that connects to a public node, completes the handshake, and asks for addresses of other peers.

Use the [`bitcoin` crate](https://docs.rs/bitcoin/latest/bitcoin/p2p/index.html), which provides (de)serialization for protocol messages.
It will save you a lot of work.

### 🔌 Connecting to the network

Open a TCP socket to `seed.bitcoin.sipa.be`, not a raw IP.

When you use a domain name in `TcpStream::connect`, the operating system performs a DNS lookup and returns a list of candidate socket addresses (IPv4 and IPv6).
Rust gives you an iterator over these addresses—explore what it contains!

Use this as a moment to practice key Rust concepts:

- What exactly does `TcpStream::connect("seed.bitcoin.sipa.be:8333")` return?
- What kind of `Result` type do you get?
- Can you inspect and print the resolved IPs before connecting?

📘 To guide your exploration, revisit:

- [Chapter 6: Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [Chapter 9: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Section 13.2: Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)

> Bonus:
> Try printing all resolved addresses before selecting one.
> What types do you see?

### 🤝 Perform the handshake

- What message does your client need to send first?
- What responses should it expect?
- How does your client know the handshake was successful?

Basic sequence:
1. Send `version`
2. Receive `version` and `verack`
3. Send `verack` to complete the handshake.
See [ProcessMessage](https://github.com/bitcoin/bitcoin/blob/dbc450c1b59b24421ba93f3e21faa8c673c0df4c/src/net_processing.cpp#L3715)

### ⏱️ Handle connection stability

- Some nodes may disconnect right after the handshake. Try using [`TcpStream::peek`](https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.peek) to detect if the socket is still open.
- What happens if your client ignores a `ping` message?

Your client should:
- Reply to `ping` with `pong`
- Send a `getaddr` message to request peers
- Receive and handle `addr` messages

### 🖨️ Print all communication

- Use simple `println!()` calls to print every message your client sends and receives.
- Don’t worry about proper logging for now—we’ll design and implement structured logging facilities in a later task.

You are going to design and implement proper logging for your program.
For now, think about how you would design:

- What information should be included in a useful log line?
- How would you distinguish between incoming and outgoing messages?
- How would you format logs to be readable, but also machine-parseable later?

Focus on the essentials:
- Understand how Bitcoin protocol messages are exchanged.
- Get comfortable working with TCP streams and binary message framing.

> 💡 This task includes all the Bitcoin protocol logic you'll implement in the seminar.
> The remaining tasks will tackle broader systems concerns like storage, concurrency, DNS, and interface design.

---

## Questions to Guide You

- What information is exchanged during the handshake?
- What assumptions does the protocol make about trust or encryption?
- What challenges arise from parsing structured messages over a raw TCP stream?
- What can you infer from the peer’s response to `getaddr`?

---

## Tools that may help

- [`dig`](https://linux.die.net/man/1/dig) – to inspect DNS seeders  
- [`nc`](https://linux.die.net/man/1/nc) – to manually probe TCP connections  

---

## References

- [Bitcoin P2P Protocol Wiki](https://en.bitcoin.it/wiki/Protocol_documentation)  
- [Bitcoin Developer Reference](https://developer.bitcoin.org/devguide/index.html)