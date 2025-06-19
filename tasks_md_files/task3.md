# Task 3: Give It Ears — Building a Command-Line Interface

> *“The only ‘intuitive’ interface is the nipple. After that, it’s all learned.”*
> — Bruce Ediger

Your client can now speak the Bitcoin protocol.
It’s time to give it **ears**—a structured way for *you* to speak to it.

In systems programming, **how** users interact with a tool is just as important as what the tool does.
Good command-line interfaces don’t just make programs usable—they make them composable, scriptable, and testable.

This idea comes straight from the **Unix philosophy**:
tools should “do one thing well” and expose that thing through a clear, consistent interface.
Bitcoin’s own tools—like `bitcoind`, `bitcoin-cli`, and `bitcoin-tx`—are built this way too.
They accept inputs through flags, arguments, and subcommands, and they behave predictably across many use cases.

You’ve already seen this principle in action through `cargo`.
It’s a complex system under the hood, but it feels approachable because:

- `cargo build` compiles your project.
- `cargo test` runs tests.
- `cargo run`, `cargo clean`, `cargo check`... all follow the same pattern.
- And `--help` works everywhere.

This simplicity is an illusion—but a very useful one.
Behind the scenes, the interface is structured with subcommands, flags, arguments, and type-driven parsing.

In this task, you’ll bring that same discipline to your own Bitcoin client using the [`clap`](https://docs.rs/clap/latest/clap/) crate.
You’ll describe your interface declaratively, and let the library take care of parsing and validation.

What’s especially interesting here—especially in a language like Rust—is that **your CLI becomes a data type**.
Using `clap`’s derive macros, you define a struct or enum that *models* your command-line interface.
Once parsed, you’re not working with raw strings—you’re working with a well-typed value that represents exactly the shape of valid input.

This is one of the great gifts of algebraic data types:
**you can model interfaces, protocols, and configurations as types**, and use the compiler to enforce their structure.
It’s an idea you’ll see again and again in Rust—and hopefully in whatever programming language you choose next.

Later, we’ll give your tool a **voice**—when we implement logging.
But for now, let’s teach it how to **listen**.

---

## Your Task: Add a CLI to Your Client

You will redesign your Bitcoin client to accept arguments and options from the command line using the [`clap`](https://docs.rs/clap/latest/clap/) crate.

### 🔍 Study an existing CLI: the Bitcoin Seeder

Start by examining the command-line interface of Pieter Wuille’s [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder):

- Read the README for a high-level overview.
- Run the seeder with `-?` and observe the available options.

Then reflect on:

- What kind of parameters does the seeder expose? Why?
- Which options are required for minimal operation? Which are optional?
- Are any configuration choices surprising to you?
- What would you keep or change if you were designing a similar tool?

This is your first step in thinking like a CLI designer: how to surface control without overwhelming the user.

> 💡 It's not a problem to be opinionated about how your tool should be used; your interface should reflect that

### 🛠️ Implement a basic CLI with `clap`

Now, start adding your own command-line interface.

1. Use `clap`'s derive macros to define your interface as a struct or enum.
2. Your CLI should include:
   - `--host` (default: `seed.bitcoin.sipa.be`)
   - `--port` (default: `8333`)
3. Stub out (i.e., accept as inputs but don't implement logic yet) additional options that will make sense later:
   - `--threads` (e.g., number of crawling threads)
   - `--logfile` (path to a future log file)
   - other ideas you’d like to explore later

You can parse these options and simply print them for now. Later tasks will wire them into the system.

> 💡 *Stubbing* means handling input now—even if it’s unused—so that when the feature is added, the interface doesn’t need to change.
> You’re anticipating the future.

📘 Review in the Rust Book:
- [Chapter 5: Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)  
- [Chapter 10.2: Traits and Derive](https://doc.rust-lang.org/book/ch10-02-traits.html)

---

## Questions to Guide You

- What kind of options should your tool support *now*? What should wait until later?
- How can you make future additions easier by stubbing options early?
- If someone else were to use your CLI, how would they know what it does?
- Does your CLI encourage correctness, or allow ambiguous or invalid usage?

---

## Why This Matters

Your program isn’t just a project—it’s a tool.
And tools need interfaces that humans (and other programs) can use reliably.

A well-structured CLI improves:
- usability,
- composability,
- maintainability,
- and discoverability (via `--help`).

And in Rust, you get to model that interface as **types**—making invalid inputs unrepresentable, and your tool safer by construction.

> 🔁 Your interface doesn’t need to be perfect now.
> As your program grows in complexity, so will its CLI.
> Future tasks will ask you to **expand** (and possibly redesign) this interface to support concurrency, persistence, logging, and more.