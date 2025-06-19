# Task 7: Serving the Network — Building Your DNS Server

> *“Naming is the first act of communication.”*

Your software now speaks, listens, remembers, and logs.
It’s time for it to **serve**.

In this task, you’ll expose your node’s knowledge of the Bitcoin network to the outside world, allowing others to query it for active peers using the **Domain Name System (DNS)**.
You’re not just crawling and persisting anymore — you’re contributing to the bootstrapping of the Bitcoin network itself.

---

## The DNS Protocol in a Nutshell

The internet works because of **names**.
Humans ask for websites like `google.com` — machines actually need IP addresses like `142.250.72.238`.
**DNS** is the system that translates between the two:
a giant, distributed, permissionless, cache-heavy database of mappings from names to addresses.

When you type `google.com` into your browser:

- Your machine sends a DNS query to a server,
- That server either knows the answer or forwards it to others,
- Eventually, you get back an IP address,
- Your machine connects via TCP/IP using that IP.

For our purposes, **we only need to support a minimal part of DNS**.
But it’s good to know the broader picture:

- **A records** map a domain name to an IPv4 address (e.g., `example.com` → `93.184.216.34`).
  This is the record type we will handle.

- **AAAA records** map a domain name to an IPv6 address. (We will not handle these.)

- **MX records** indicate where email should be delivered for a domain (Mail Exchange servers).

- **CNAME records** provide aliasing — they map one name to another name.

- **TXT records** allow arbitrary text metadata to be attached to a domain (e.g., for email verification).

There are many others, but these are the most commonly used.
For this project, **we will only answer queries for A records** and ignore or gracefully reject others.

---

### How DNS Messages Are Structured

Despite seeming magical, DNS queries and responses are simple in structure.
Every DNS message has:

- **A header**: 12 bytes containing metadata like ID, flags (e.g., query/response), question count, answer count, etc.
- **A question section**: containing the domain name being queried, the type of record requested (A, AAAA, MX, etc.), and the query class (almost always IN for Internet).
- **An answer section** (only in responses): containing the resource records answering the query (e.g., IP addresses).

DNS packets are **binary-encoded**, not plain text.
They require parsing fields carefully — but for simple A record queries, the layout is compact and manageable.

For full details, you can consult:

- [RFC 1034: Domain Names — Concepts and Facilities](https://datatracker.ietf.org/doc/html/rfc1034)
- [RFC 1035: Domain Names — Implementation and Specification](https://datatracker.ietf.org/doc/html/rfc1035)

(But don’t get lost — you only need a small subset to succeed in this task.)

---

## Your Task: Build a Minimal DNS Server

You’ll write a new component that:

- Listens for DNS queries,
- Matches A record queries for a specific domain (e.g., `seed.example.com.`),
- Responds with a list of **up to 10 random reachable peers**.

---

### 🛠️ 1. Bind to UDP Port 53

- Use `tokio::net::UdpSocket` to bind to port 53 on your machine.
- Be aware: on most OSes, binding to privileged ports (below 1024) requires root/admin permissions.
  **Suggestion:** for testing, you can bind to an unprivileged port like 1053, then use `dig` with `-p 1053`.

Example:

```bash
dig @127.0.0.1 -p 1053 seed.example.com
```

---

### 📦 2. Parse basic DNS queries

You don't need a full DNS parser. Focus on:

- Receiving raw UDP packets,
- Parsing the header to check if it’s a **standard query**,
- Inspecting the question section to see:
  - Is the query asking for an **A record**?
  - Is the query class **IN** (Internet)?

If it's an unsupported query type (e.g., AAAA, MX, CNAME):

- Ideally, respond with a DNS error code:
  - RCODE = 4 ("Not Implemented") as defined in [RFC 1035, section 4.1.1](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1).
- Alternatively, you may simply ignore the packet (but logging it is recommended).

If you want help parsing, consider lightweight crates like [`trust-dns-proto`](https://docs.rs/trust-dns-proto/latest/trust_dns_proto/),
or manually parse the small subset needed — it’s a great exercise in working with binary protocols.

---

### 📡 3. Reply with random known peers

- When a valid A query arrives:
  - Randomly select up to 10 **active** peers from your database,
  - Build a DNS response packet containing their IP addresses,
  - Send it back to the requester.

If you have fewer than 10 peers, return as many as you have.
If you have none, respond with a DNS "no answers" response (empty answer section).

Remember to copy relevant fields (e.g., the Transaction ID) from the original query into your response.

---

### 🔒 4. Handle concurrency carefully

Your database is now updated asynchronously by crawl workers.
When your DNS server reads from it, make sure the access is **thread-safe**.

DNS serving must **never block** your other activities — it must coexist gracefully with crawling, persistence, and logging.

Use appropriate synchronization (e.g., `Arc<RwLock<_>>`, `Arc<Mutex<_>>`) if necessary, and prefer read-only access when possible.

---

## Questions to Guide You

- How do you model a minimal DNS response with multiple A records?
- What fields are needed in the response packet (e.g., transaction ID, flags, counts, answers)?
- How will you randomly select peers fairly and efficiently?
- How will you structure database reads to avoid race conditions or crashes?
- How will you log DNS requests and responses for observability?
- What will your DNS server do when it receives a query it doesn't support?

---

## Why This Matters

By completing this task, your node stops being just a consumer of the network — it becomes a **provider of network infrastructure**.

Running a reliable DNS seeder is a service to Bitcoin nodes who need help bootstrapping.
It’s also a test of real systems engineering:
handling network packets, managing concurrency, and exposing a public-facing API without breaking your internal architecture.

> In the final task, you’ll extend your tool even further — adding an RPC interface for more flexible interaction.