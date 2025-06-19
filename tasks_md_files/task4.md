# Task 4: Remember What You Learn — Storing Peer Information

> *“Memory is the residue of thought.”*
> — Daniel T. Willingham

Your client now speaks the Bitcoin protocol.
It connects to peers, exchanges messages, and discovers others.
But the moment you shut it down, it forgets everything it learned.
In this task, we begin the process of giving it a memory—of teaching it how to *remember*.

But before you write a single line of code, take a step back and think like a systems designer:

### 🤔 What is a peer?

Not in the abstract—not “a computer running Bitcoin somewhere”—but from the perspective of *your software*.
If your goal is to help nodes bootstrap into the network, what information about a peer matters?

Start modeling the domain.
A peer, at minimum, has an **address**—typically an IP address and a port.
This could be represented by a standard `SocketAddr` in Rust, or perhaps by separating the IP (which could be v4 or v6) and port fields explicitly for clarity and extensibility.

But that’s just the surface.
Consider the **temporal dimension**:
the Bitcoin network is highly dynamic, so knowing *when* you last interacted with a peer is often as important as who the peer is.
You might store timestamps for the last time you successfully connected, the last failed attempt, or when you first heard about the peer from others.
These can be stored as `SystemTime` or converted to UNIX timestamps (`u64`), depending on your needs.

Then there’s **connectivity status**:
was the last connection successful?
Do you want to mark peers that consistently fail or misbehave?
These can be modeled with an enum like:

```rust
enum PeerStatus {
    NeverTried,
    ConnectedRecently,
    Unreachable,
    Banned,
}
```

You should also consider whether a peer supports particular Bitcoin features—like compact blocks, address relay, or bloom filters.
These are usually advertised in the protocol via **service flags**, and you can model them in your code as a bitmask or a set of typed constants.

If you include this information in your peer model now, you’ll be able to do something powerful later:
**expose it through your CLI**.
Imagine adding a `--filter-service` flag to your DNS server interface that only returns peers supporting a given feature.
By connecting your data model to your command-line interface, you’re making your tool both smarter and more usable.

### 🧱 What kind of state does that require?

- A peer might be represented as a struct with fields for its network address, timestamps, and current status.
- A list of peers might live in a `HashMap<SocketAddr, PeerInfo>`, or in a `Vec` if order matters more than lookup speed.
- You’ll need a way to serialize and persist this state—JSON, CSV, or even a line-per-peer plaintext format is fine for now.
  But also consider: **would it make sense to use a proper embedded database**, like SQLite or sled? This may seem like a complication, but could offer real advantages in terms of robustness, performance, and flexibility for querying peers later on.

By building a model that reflects the real-world behavior of peers, you’re laying the groundwork for smarter logic, more robust decisions, and more useful behavior down the line.

This isn’t just about storage—it’s about shaping your program’s **understanding** of the network.

In this task, you’ll implement:

- an in-memory representation of peer state,
- serialization and deserialization to save/load it from disk,
- and basic logic to update the state as your program runs.

It doesn’t need to be perfect.
But it needs to be designed with care.

---

## Your Task: Add in-memory and persistent peer storage

Design a data model that tracks Bitcoin peer addresses and their status.
This isn’t just a matter of saving state—it’s about helping your client become more autonomous.

### 🔁 Revisit what you learned

In [Task 1](#), you explored Pieter Wuille’s [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder) and examined its use of multiple threads and internal data structures.
Now’s a good time to reflect on that investigation:

- How did the Seeder organize peer data?
- What kind of statuses did it track?
- What filtering or categorization logic did it support?
- Were you satisfied with how it was structured? What would you do differently?

Bring that critical eye into your own design.
Your goal is to create something better—simpler, safer, and more extensible.

### 📦 Define your peer model

Design a `struct` that represents what your software *knows* about a peer.
Include:

- Network address (IP and port)
- Timestamps for last seen, last connected, etc.
- Status indicators (e.g., banned, unreachable, healthy)
- Optional: supported service flags (for filtering later)

### 🧠 Maintain a peer list in memory

- Store peer records in a suitable collection—`HashMap` is a good default.
- Update the in-memory state as you connect to nodes and receive `addr` messages.

### 💾 Persist to disk

- Serialize your peer list at shutdown using JSON, CSV, or another format.
- On startup, load it back into memory to resume operation.

> 💡 Tip: Use `serde` to make this simple and ergonomic.

### 🌐 Reconnect using the database

Your client should now try to reconnect using **its own database first**, rather than calling out to a DNS seeder every time it starts.

Suggested logic:

1. On startup, load the peer list from disk.
2. Attempt to connect to a few peers from the list.
3. If all attempts fail, fall back to a DNS seeder to bootstrap.
4. Integrate any new addresses into the existing in-memory state.

This is your software's graduation moment:
it now carries **its own memory** of the network and tries to grow from that.
It only asks for outside help when absolutely necessary.

---

## Questions to Guide You

- **How do you define a peer?** What fields are essential for your software to make useful decisions about them?
- **Which events should update your peer state?** When do you overwrite vs. retain historical info?
- **How will you structure your in-memory data?** Are you prioritizing fast lookup, iteration order, grouping by subnet, or something else?
- **What makes a peer “valuable” or “active”?** How can your data model capture that judgment?
- **What’s a good persistence format for now?** Will JSON suffice, or are you tempted to try a more powerful format like SQLite?
- **When should you fall back to a DNS seeder?** How many failed reconnection attempts is “enough” before deciding you’re isolated?
- **Is your design resilient to crashes or partial writes?** What might go wrong if your file is corrupted?
- **How will you grow this system later?** Can you imagine needing new fields or logic without rewriting the whole thing?

---

## Bonus Challenges

- Group peers by subnet or other logical criteria.
- Use `serde` to serialize/deserialize your peer list automatically.
- Try using SQLite or `sled` instead of a flat file format.
- Design CLI flags that query or filter the peer database (e.g., by service flags or activity level).

---

## Why This Matters

Your DNS seeder will eventually serve peer addresses to others.
That requires tracking which peers are reachable, useful, or broken.
Without persistence, your software resets every time it runs—making your service unreliable.

This task helps you begin thinking like a systems programmer:

- Where does state live?
- How should it be organized?
- And how do you keep it durable over time?

> 🧱 You are laying the groundwork for the rest of the project.
> The structure you build here will shape how concurrency, DNS, and logging interact in future tasks.