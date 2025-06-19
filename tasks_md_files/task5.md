# Task 5: From Soloist to Conductor — Introducing Concurrency

> *“The conductor must not only play the instruments—he must also hear them all.”*

So far, you've built something remarkable:
a Bitcoin client that speaks the protocol, connects to peers, remembers what it learns, and accepts commands from a structured CLI.
You’ve also designed a data model to persist that knowledge across runs.

But everything your program does—all the connections, all the processing, all the persistence—runs in a single thread.
That thread, the `main` function, is doing everything:
it listens, it talks, it reads, it writes.
It is the soloist in your symphony.

This works. But it doesn't scale.

---

### 🚧 What does it mean not to scale?

In this case, it means that as soon as your program is waiting for one thing—say, a peer to respond—it’s not doing anything else.
It means you can’t easily connect to multiple peers at once.
It means your logging might block your networking.
It means your DNS server (soon!)
would hang while waiting on a disk write.

And most importantly:
it means all the beautiful logic you've built so far is **tightly coupled**—difficult to separate, test, reuse, or evolve.

---

### 🪞 Let’s pause and reflect

Take a step back and look at your artifact:

- Where are the boundaries between concerns?
- What roles can you identify in your system?
- What components could operate independently if given the chance?

You’ve probably already implemented or stubbed out:

- A **CLI interface** for input configuration
- **P2P networking logic** for connecting and speaking to Bitcoin nodes
- **Persistence logic** to store and retrieve peer information
- Soon, **DNS service logic** to respond to external requests
- And potentially a **logging mechanism** (coming up!)

Each of these responsibilities has its own rhythm, its own tempo.
And if you want to coordinate them without chaos, you need to move from a solo performance to **a system of communicating threads**.

---

### 🧵 What might these threads look like?

#### 🧭 The peer connection manager

This thread is responsible for selecting peers from your database and initiating connections.
It schedules work, tracks connection attempts, and maybe uses basic heuristics to avoid retrying bad nodes too often.

#### 🔌 The crawl worker(s)

These async tasks establish TCP connections with peers, perform the handshake, and request more addresses (`getaddr`).
Each task handles one peer and reports the outcome back to the system.

#### 🗃️ The database writer

A dedicated thread or async task that receives structured messages and updates the peer database accordingly.
It batches and writes to disk when needed.

#### 🧾 The logger (optional for now)

This will come later, but you might start imagining how log messages could be routed to a centralized service that doesn't block any of the logic above.

---

### ⚠️ A word of caution

This task might feel more challenging than previous ones—not necessarily because of the threading itself, but because of the **shape of the code you’ve already written**.
If your logic is tightly coupled, if responsibilities are blurred, or if there are no clear boundaries between components, splitting the program into threads will feel messy.

But if you’ve been thinking in terms of **interfaces**, **responsibilities**, and **data flow**, this transformation will be much smoother.

This is a good moment to reflect:
did you design your peer handling logic, your persistence layer, and your CLI interface as modules that talk through clear interfaces?
If not, now is a great time to refactor toward that.

You're not just learning Rust or Bitcoin here—you’re learning how to design software that grows.

---

## Your Task: Refactor for Concurrency with Tokio

You’ll transition your client from a sequential, blocking architecture to a concurrent, non-blocking one using the [`tokio`](https://tokio.rs/) runtime and Rust’s native `async/await` support.

This will unlock the ability to **connect to multiple peers simultaneously**, and **delegate responsibilities** to independent threads, such as writing to the peer database.

---

### ✅ 1. Replace blocking networking with Tokio

Refactor your connection logic to use asynchronous networking primitives:

- Replace `std::net::TcpStream` with `tokio::net::TcpStream`.
- Convert your crawl logic into `async fn crawl_peer(addr: SocketAddr)`.
- Use `.await` to handle I/O instead of blocking calls.

🧭 **References**:
- [The Rust Book — Chapter 14.3: Working with Futures and async/await](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Tokio async TCP docs](https://docs.rs/tokio/latest/tokio/net/index.html)

> 🔎 You’ll likely hit challenges passing variables like database handles, loggers, or peer data between `async` functions.
> This is a great moment to revisit how **ownership, borrowing, and lifetimes** work in Rust.

📘 **Ownership Primer**:  
- [The Rust Book — Chapter 4: Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)

---

### 🔄 2. Connect to multiple peers in parallel

- Spawn **at least 3 simultaneous calls** to `crawl_peer()` using `tokio::spawn`.
- Use `join!`, `join_all`, or `FuturesUnordered` to manage and await these tasks.
- Ensure each peer connection is handled independently.

This is your first true moment of **parallel peer crawling**.

---

### 🗃️ 3. Parallelize your database access

This step is **mandatory**, as it will be crucial in later tasks.

- Move your database logic into a dedicated thread or async task.
- Use a **channel** (e.g., `tokio::sync::mpsc`) to send structured messages from crawling tasks to the database logic.
- Define your message types clearly, e.g.:

```rust
enum DbCommand {
    InsertPeer(Peer),
    MarkUnreachable(SocketAddr),
}
```

📘 For working with shared state, you may need to use `Arc`, `Mutex`, or interior mutability. See:

- [The Rust Book — Chapter 15.3: Shared-State Concurrency](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)

Design your interface so that each component can operate independently but communicate cleanly.
Think **actor model**, even if implemented simply.

---

## Questions to Guide You

- How will you manage ownership when moving data between threads and tasks?
- Can each of your components (crawlers, database, etc.) be reused or replaced independently?
- How are failures reported and handled? What happens if a peer fails to respond?
- How will you verify that peer crawls are running concurrently? (Think: logs or timing.)
- Is your message-passing interface clear and extensible? Would someone else understand how to use it?