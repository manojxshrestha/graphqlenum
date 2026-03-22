<h1 align="center">
  <br>
  <a href="https://github.com/manojxshrestha/">
    <img src="https://github.com/user-attachments/assets/b7534e37-a4de-45c9-ae15-a556211fe11f" alt="graphqlenum" width="600">
  </a>
  <br>
  graphqlenum
  <br>
</h1>

<p align="center">
Automated GraphQL security testing for security researchers
</p>

<div align="center">

[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/manojxshrestha/graphqlenum/blob/main/LICENSE)
[![GitHub Repo](https://img.shields.io/badge/repo-GitHub-black?logo=github)](https://github.com/manojxshrestha/graphqlenum)
[![Issues](https://img.shields.io/github/issues/manojxshrestha/graphqlenum)](https://github.com/manojxshrestha/graphqlenum/issues)
[![Stars](https://img.shields.io/github/stars/manojxshrestha/graphqlenum?style=social)](https://github.com/manojxshrestha/graphqlenum)

</div>

---

**graphqlenum** is a bash-based security auditing tool for bug bounty hunters and penetration testers. It automates GraphQL security testing by enumerating paths to sensitive data types, discovering hidden endpoints, and extracting data from vulnerable GraphQL APIs.

### Features

- **Path Enumeration** - Find all possible paths to any type in the GraphQL schema
- **Sensitive Type Detection** - Auto-detect User, Payment, Order, Address, Token and other sensitive types
- **Data Extraction** - Query and retrieve real data from any available field
- **Auto-Enum** - Automatically query all available fields and extract data
- **Offline Analysis** - Work with previously saved schema files
- **Mutation Scanning** - Include mutation paths in enumeration with `-m` flag
- **Auto-Save** - All results automatically saved to `data/` directory

---

## Installation

```bash
git clone https://github.com/manojxshrestha/graphqlenum.git
cd graphqlenum
chmod +x graphqlenum install.sh
./install.sh
```

### Requirements

- `curl` - For making HTTP requests
- `jq` - For JSON parsing

---

## Quick Start

```bash
# 1. List sensitive types and query fields
./graphqlenum https://api.example.com/graphql

# 2. Find all paths to User type
./graphqlenum https://api.example.com/graphql User -m

# 3. Extract data from a specific field
./graphqlenum https://api.example.com/graphql -d users

# 4. Auto-extract ALL available data
./graphqlenum https://api.example.com/graphql -a
```

---

## Detailed Usage

### Basic Commands

| Command | Description |
|---------|-------------|
| `./graphqlenum https://api.com/graphql` | List all sensitive types and query fields |
| `./graphqlenum https://api.com/graphql User` | Find paths to User type |
| `./graphqlenum https://api.com/graphql User -m` | Find paths to User type (including mutations) |
| `./graphqlenum https://api.com/graphql -d users` | Get data from the `users` field |
| `./graphqlenum https://api.com/graphql -a` | Auto-retrieve ALL query data |
| `./graphqlenum -i schema.json User` | Use saved schema file |

### Flags

| Flag | Description |
|------|-------------|
| `-m` | Include mutation paths in enumeration |
| `-d <field>` | Get and display data from a specific field |
| `-a` | Auto-retrieve data from ALL available query fields |
| `-i <file>` | Use a previously saved schema file |

---

## Bug Bounty Hunting Guide

### 1. Initial Reconnaissance

Start by enumerating the GraphQL schema to discover what's available:

```bash
./graphqlenum https://target.com/graphql
```

**What you'll find:**
- Sensitive types: User, Payment, Order, Customer, Account, Token, etc.
- Query fields: available endpoints you can query
- Data saved to `data/target_com_sensitive.txt` and `data/target_com_queries.txt`

### 2. Finding Sensitive Data Paths

Discover how to reach sensitive types through the API:

```bash
./graphqlenum https://target.com/graphql User -m
```

**Example output:**
```
Found 12 ways to reach "User":
- QueryRoot (viewer) -> User
- QueryRoot (user) -> User
- QueryRoot (users) -> UserConnection -> UserEdge -> User
- QueryRoot (admin) -> Admin -> User
```

**Bug Bounty Tips:**
- Look for unexpected paths to sensitive data
- Check if you're authorized to access admin-only paths
- Enumerate all types: User, Customer, Payment, Order, Admin, Token, etc.

### 3. Extracting Data

Query specific fields to extract data:

```bash
# Get users
./graphqlenum https://target.com/graphql -d users

# Get orders
./graphqlenum https://target.com/graphql -d orders

# Get payments
./graphqlenum https://target.com/graphql -d payments
```

### 4. Full Data Extraction

Automatically extract data from all query fields:

```bash
./graphqlenum https://target.com/graphql -a
```

This will:
- Query every available field
- Save all responses to `data/` directory
- Help you discover data exposure vulnerabilities

### 5. Offline Analysis

Save schemas for offline testing or to compare changes:

```bash
# First, save the schema
curl -s -X POST "https://target.com/graphql" \
  -H "Content-Type: application/json" \
  -d '{"query":"{ __schema { queryType { name } mutationType { name } types { kind name fields { name type { kind name ofType { kind name ofType { kind name ofType { kind name } } } } } } } }"}' > schema.json

# Then analyze offline
./graphqlenum -i schema.json User -m
./graphqlenum -i schema.json Payment
./graphqlenum -i schema.json -a
```

---

## Common Vulnerabilities to Look For

### Information Disclosure

```bash
# Find all data paths
./graphqlenum https://target.com/graphql -a
```

Look for:
- Exposed PII (emails, phones, addresses)
- Internal IDs and system information
- Pricing and business data

### IDOR (Insecure Direct Object References)

```bash
# Find object access patterns
./graphqlenum https://target.com/graphql User -m
./graphqlenum https://target.com/graphql Order -m
```

Look for:
- Predictable ID enumeration
- Missing authorization checks
- Direct access to other users' data

### Mass Assignment

```bash
# Query different object types
./graphqlenum https://target.com/graphql -d user
./graphqlenum https://target.com/graphql -d account
```

Look for:
- Writable fields you shouldn't access
- Hidden admin fields
- Privilege escalation vectors

### GraphQL-specific Issues

```bash
# Find all mutation paths
./graphqlenum https://target.com/graphql User -m
```

Look for:
- Missing rate limiting
- Introspection enabled in production
- Debug mode enabled
- Stack traces in errors

---

## Output Files

All results are automatically saved to the `data/` directory:

| File Pattern | Description |
|--------------|-------------|
| `*_sensitive.txt` | List of sensitive types found |
| `*_queries.txt` | All available query fields |
| `*_paths.txt` | Path enumeration results |
| `*_*.json` | Raw data from field queries |

### Example

```bash
$ ./graphqlenum https://api.example.com/graphql -a

[*] Found 34 query fields
[*] Extracting data from all fields...
[+] api_example_com_users.json
[+] api_example_com_orders.json
[+] api_example_com_payments.json
...

$ ls data/
api_example_com_sensitive.txt
api_example_com_queries.txt
api_example_com_users.json
api_example_com_orders.json
api_example_com_payments.json
```

---

## Real-World Examples

### Example 1: E-commerce Platform

```bash
# Find paths to payment data
./graphqlenum https://shop.example.com/graphql Payment -m

# Extract customer orders
./graphqlenum https://shop.example.com/graphql -d customerOrders

# Find all sensitive types
./graphqlenum https://shop.example.com/graphql
```

### Example 2: Social Media API

```bash
# Find user data paths
./graphqlenum https://api.social.example.com/graphql User -m

# Get user profile
./graphqlenum https://api.social.example.com/graphql -d profile

# Auto-extract all public data
./graphqlenum https://api.social.example.com/graphql -a
```

### Example 3: Banking/Fintech

```bash
# List financial data types
./graphqlenum https://api.bank.example.com/graphql

# Find account access paths
./graphqlenum https://api.bank.example.com/graphql Account -m

# Check for transaction disclosure
./graphqlenum https://api.bank.example.com/graphql -d transactions
```

---

## Tips for Bug Bounty Hunters

1. **Always check for GraphQL first**
   ```bash
   # Common endpoints
   /graphql
   /api/graphql
   /api/v1/graphql
   /graphql/schema
   ```

2. **Use with other tools**
   ```bash
   # Combine with nmap
   nmap -p 443 --script=graphql-introspection target.com

   # Use with ffuf for fuzzing
   ffuf -u https://target.com/graphql -w wordlist.txt
   ```

3. **Document everything**
   ```bash
   # Create target directory
   mkdir -p data/target.com
   cd data/target.com

   # Run all checks
   ../../../graphqlenum https://target.com/graphql -a
   ```

4. **Check for common misconfigurations**
   - Introspection enabled in production
   - Missing rate limiting
   - No authentication on sensitive queries
   - Information disclosure in error messages

5. **Bypass common protections**
   ```bash
   # Try with different headers
   curl -X POST https://target.com/graphql \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer ..." \
     -H "X-Forwarded-For: 127.0.0.1"

   # Try batching
   curl -X POST https://target.com/graphql \
     -d '[{"query":"{ users { id } }"},{"query":"{ admin { id } }"}]'
   ```

---

## Troubleshooting

### Introspection Failed

Some endpoints disable introspection. Try:
```bash
# Check if introspection is available
curl -X POST https://target.com/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"{ __schema { queryType { name } } }"}'

# If it fails, the endpoint might:
# - Disable introspection
# - Require authentication
# - Use a different endpoint
```

### Query Depth Issues

Some schemas have deep nesting. The tool automatically handles:
- Non-null types (`!`)
- List types (`[...]`)
- Nested objects up to 10 levels

### Missing Fields

If you see "Field not found" errors:
- The field might require arguments
- Check the schema manually
- Try a different field

---

## Contributing

Contributions are welcome! Please feel free to submit a [Pull Request](https://github.com/manojxshrestha/graphqlenum/pulls).

---

## Support

If this tool helps you in your security research and bug bounty, I'd appreciate a coffee, haha:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-manojxshrestha-orange?logo=buymeacoffee)](https://buymeacoffee.com/manojxshrestha)

---

## License

This project is open-source and available under the [MIT License](LICENSE).
