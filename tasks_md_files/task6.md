# Task 6: Let It Speak — Adding Logging to Your Concurrent System

> *“If a tree falls in a forest and no one is around to hear it, does it make a sound?”*
> — Philosophical proverb (and every developer debugging a silent crash)

Your system is now alive:
it listens, it speaks, it stores, it connects.
You've given it a voice in the form of a CLI, and parallel limbs through concurrency.
But it's still mostly silent — especially when things go wrong.

In this task, your software will learn to **speak back**.
You’ll give it a logging mechanism that can be used by every part of the system — a central service that accepts structured messages and records them clearly, reliably, and without disrupting the rest of your application.

---

## Why Logging Matters

As your client grows, it becomes harder to understand what it's doing at any given moment.
Imagine you start your program and try to connect to several peers — but nothing seems to happen.
Is the issue with DNS resolution?
Did your client fail to connect because of a malformed handshake?
Did a thread panic and exit silently?
Did you forget to `.await` an async task?

Without any kind of feedback, you're left guessing.
You stare at an empty terminal, unsure whether your program is stuck, idle, or simply finished.

In concurrent systems, **observability is everything**.
You need a way to see that your tasks are alive — that they are moving, failing, recovering.
You want to trace which parts of the system are active and which ones are blocked.
You want to understand when a peer was crawled, what the result was, whether the data was persisted or discarded.
You want to identify failure modes and performance bottlenecks, not by trial and error, but by watching your system talk about what it’s doing.

And you want all of that without disrupting the flow of the application.

As you build your logging service, think carefully:
**not all events are equally important**.
Some events simply inform that a step succeeded.
Others hint at potential problems.
And some signal outright failure.
Part of building a real-world logging system is classifying events by **severity** — distinguishing information, warnings, and errors — and giving users control over what they want to see.

In this task, you won’t just log events.
You’ll model them with care, including their importance, and expose **verbosity control** through your CLI.

---

## Your Task: Add a Logging Thread

You’ll create a **dedicated thread or async task** responsible for receiving and recording structured log messages.

This is not just about adding `println!()` everywhere.
This is about **designing a small model of your system's behavior**:
you will use types — enums and structs — to represent meaningful events, and you will log those events consistently.

Think of this as building the "narrator" of your program.

---

### 🧵 1. Design a centralized logging service

Define a `LogMessage` struct (or a similar structure) that covers two aspects:

- **The event itself** (e.g., Connected to a peer, Failed connection, Discovered new peer, Wrote to disk)
- **The severity level** of the event (e.g., Trace, Debug, Info, Warn, Error)

Example:

```rust
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

enum Event {
    Connected(SocketAddr),
    FailedConnection(SocketAddr, String),
    PeerDiscovered(SocketAddr),
    SavedToDisk(usize),
    Custom(String),
}

struct LogMessage {
    level: LogLevel,
    event: Event,
}
```

Spawn a task or thread that owns a `tokio::mpsc::Receiver<LogMessage>`.
Whenever a message is received, print it clearly and concisely using `println!()` or `eprintln!()`.

> 💬 Later, you could easily swap the `println!()` calls for structured logging libraries like [`tracing`](https://docs.rs/tracing/) or [`log`](https://docs.rs/log/).

---

### 📣 2. Emit log messages from across the system

Identify meaningful points in your program and emit `LogMessage`s for them:

- When a peer connection succeeds or fails,
- When a new peer address is discovered,
- When the database writes to disk,
- When important internal operations succeed or error.

Model your messages carefully. Avoid logging arbitrary strings; **log structured events** that can be filtered, categorized, or reformatted later without changing the business logic.

---

### ⚙️ 3. Add verbosity control to your CLI

Extend your CLI interface (using `clap`) to accept a new option like:

```bash
--verbosity debug
```
or

```bash
--verbosity info
```

At startup, your program should parse this flag and determine a minimum severity level to display.
Messages below the chosen verbosity level should be ignored (or printed optionally).

This will allow users to run your tool in quiet mode (only seeing errors and warnings) or verbose mode (seeing detailed internal events).

---

## 🌟 Bonus Challenge: Fine-grained verbosity control

In a more sophisticated system, you might want **different verbosity levels for different components**.

For example:

- See only `info` and `warn` messages from the networking layer,
- But see full `trace` logs for database writes and updates,
- Or suppress all `debug` noise from DNS queries while debugging peer crawling.

You can start thinking about how to model this:

- Assign **categories** or **subsystems** to each `LogMessage` (e.g., Networking, Database, CLI, DNS).
- Allow users to configure **different verbosity levels** per category.
- Filter incoming log messages based on both their severity and their category settings.

You don’t need to implement this fully now — just **structure your code in a way that would allow it** later, without a huge rewrite.

This is a natural extension toward real-world structured logging frameworks like [`tracing`](https://docs.rs/tracing/latest/tracing/), which organize logs into spans, events, and fields.

> 🧠 Fine-grained observability is one of the main reasons complex distributed systems survive in production.

---

## Questions to Guide You

- What events are worth logging? What signals valuable state changes or error conditions?
- How will you model different kinds of loggable events cleanly?
- Should the logger block on printing? (Hint: no — that's why it runs independently.)
- How should verbosity filtering work? Where should it be enforced (at the sender, at the receiver, or at display time)?
- Could your logging design later adapt to file logs, JSON logs, or remote telemetry without rewriting the business logic?

---

## Why This Matters

Logging is not decoration — it is **interface design** for human operators.
It’s what you (or someone else) will read when your program misbehaves, when performance is slow, or when you need to understand what happened three minutes ago.

Good logging helps you debug, operate, and trust your system.
This task brings your client one step closer to being a real-world service.

> In the next task, you’ll begin serving peer data to the outside world via DNS.
> It will be your first externally-facing feature — and logging will be your only way to know if it's working.